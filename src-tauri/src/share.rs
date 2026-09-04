use crate::diagnose::{self, AssetReport};
use crate::network::{current_network, NetworkInfo};
use crate::proxy::{self, DevServer};
use crate::watch::{self, SiteWatcher};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use percent_encoding::percent_decode_str;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

const ACCESS_ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
const DEFAULT_PORT: u16 = 4173;
/// 超过这个时长没有请求就认为设备已离开。注入脚本每 700ms 轮询一次，
/// 所以在线设备一定会持续刷新时间戳；这个阈值只用来清掉真正走掉的设备。
const DEVICE_TTL_MS: u64 = 15_000;

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Mutex<InnerState>>,
    /// 站点版本号。文件监听器递增它，注入页面的脚本轮询它决定是否刷新。
    ///
    /// 单独放在 `AtomicU64` 而不是 `InnerState` 里，是为了让监听回调只持有
    /// 这个计数器 —— 否则回调持有 `Arc<Mutex<InnerState>>`，而 `InnerState`
    /// 又持有 watcher，形成引用循环。
    revision: Arc<AtomicU64>,
}

pub struct InnerState {
    pub selected_root: Option<PathBuf>,
    pub entry: Option<String>,
    pub selected_ip: Option<String>,
    /// 反向代理目标端口。`Some` 时所有请求转发给本机 dev server。
    proxy_port: Option<u16>,
    preview: Option<RunningServer>,
    share: Option<ShareSession>,
    watcher: Option<SiteWatcher>,
    /// 跨分享会话生效的设备黑名单。设备当前以 IP 作为唯一标识。
    blacklisted_ips: HashSet<String>,
    blacklist_path: Option<PathBuf>,
    /// 共享代次。过期定时器带着自己的代次醒来，代次不匹配就说明
    /// 用户已经停止并重开过共享，此时必须什么都不做。
    generation: u64,
}

struct RunningServer {
    port: u16,
    shutdown: Option<oneshot::Sender<()>>,
    join: JoinHandle<()>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectedDevice {
    pub ip: String,
    pub name: String,
    /// `phone` / `tablet` / `desktop` / `unknown`，前端据此选图标。
    pub kind: String,
    pub last_seen: u64,
}

struct ShareSession {
    code: String,
    cookie: String,
    expires_at: Option<u64>,
    lan_ip: String,
    devices: HashMap<String, ConnectedDevice>,
    /// 仅本次分享有效的访问屏蔽。
    blocked_ips: HashSet<String>,
    server: RunningServer,
}

#[derive(Clone)]
struct HttpContext {
    state: AppState,
    require_code: bool,
}

#[derive(Debug, Serialize)]
pub struct SiteInfo {
    pub name: String,
    pub root: String,
    pub entry: String,
    pub html_entries: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ShareStatus {
    pub active: bool,
    pub url: Option<String>,
    pub preview_url: Option<String>,
    pub access_code: Option<String>,
    pub expires_at: Option<u64>,
    pub connections: usize,
    pub devices: Vec<ConnectedDevice>,
    pub entry: Option<String>,
    pub root: Option<String>,
    pub port: Option<u16>,
    /// `static` 直出磁盘文件，`proxy` 转发到本机 dev server。
    pub mode: String,
    pub proxy_port: Option<u16>,
    pub network: NetworkInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartShareInput {
    pub expiry_minutes: Option<u64>,
    pub access_protection: Option<bool>,
    pub network_ip: Option<String>,
}

impl Default for InnerState {
    fn default() -> Self {
        Self {
            selected_root: None,
            entry: None,
            selected_ip: None,
            proxy_port: None,
            preview: None,
            share: None,
            watcher: None,
            blacklisted_ips: HashSet::new(),
            blacklist_path: None,
            generation: 0,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(InnerState::default())),
            revision: Arc::new(AtomicU64::new(1)),
        }
    }

    /// 黑名单是安全策略，不应只活在一轮进程里。读取失败时按空名单启动，
    /// 不让一个坏的偏好文件阻止整个应用启动。
    pub fn configure_blacklist_storage(&self, directory: PathBuf) {
        let path = directory.join("device-blacklist.json");
        let entries = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok())
            .unwrap_or_default();
        if let Ok(mut inner) = self.inner.lock() {
            inner.blacklisted_ips = entries
                .into_iter()
                .filter(|ip| !ip.trim().is_empty())
                .collect();
            inner.blacklist_path = Some(path);
        }
    }

    fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }

    /// 安装（或替换）站点监听器。改动到来时递增版本号，页面轮询后自行刷新。
    fn rewatch(&self, root: &Path) {
        let counter = self.revision.clone();
        let watcher = watch::watch(root, move || {
            counter.fetch_add(1, Ordering::Relaxed);
        });
        // 版本号也要跳一次：换站点本身就意味着页面内容变了。
        self.revision.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut inner) = self.inner.lock() {
            inner.watcher = watcher;
        }
    }
}

fn is_skipped_dir(name: &str) -> bool {
    // 与文件监听共用同一份忽略清单，避免"能扫到但监听不到"这类不一致。
    // 注意清单里没有 dist / build / out —— 构建产物目录正是最常被共享的站点根。
    watch::is_ignored_dir(name)
}

fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// 入口候选与遍历上限。原先 20 / 80 太小，稍大的站点会漏掉 index.html。
const MAX_HTML_ENTRIES: usize = 50;
const MAX_WALK_VISITS: usize = 4000;

pub fn list_html_entries(root: &Path) -> Vec<String> {
    let has_index = root.join("index.html").is_file() || root.join("index.htm").is_file();
    list_html_entries_limited(root, if has_index { 0 } else { 2 })
}

pub fn list_html_in_dir(root: &Path) -> Vec<String> {
    list_html_entries_limited(root, 0)
}

fn list_html_entries_limited(root: &Path, max_depth: usize) -> Vec<String> {
    let mut found = Vec::new();
    let mut visits = 0usize;
    fn walk(
        dir: &Path,
        root: &Path,
        found: &mut Vec<String>,
        visits: &mut usize,
        depth: usize,
        max_depth: usize,
    ) {
        if depth > max_depth || found.len() >= MAX_HTML_ENTRIES || *visits >= MAX_WALK_VISITS {
            return;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            if found.len() >= MAX_HTML_ENTRIES || *visits >= MAX_WALK_VISITS {
                break;
            }
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if is_hidden(&name) || is_skipped_dir(&name) {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            *visits += 1;
            if file_type.is_dir() {
                subdirs.push(entry.path());
                continue;
            }
            let lower = name.to_ascii_lowercase();
            if lower.ends_with(".html") || lower.ends_with(".htm") {
                if let Ok(rel) = entry.path().strip_prefix(root) {
                    found.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        if depth < max_depth {
            for path in subdirs {
                walk(&path, root, found, visits, depth + 1, max_depth);
            }
        }
    }
    walk(root, root, &mut found, &mut visits, 0, max_depth);
    found.sort_by(|a, b| {
        let rank = |value: &str| {
            let lower = value.to_ascii_lowercase();
            if lower == "index.html" {
                0
            } else if lower.ends_with("/index.html") {
                1
            } else {
                2
            }
        };
        rank(a).cmp(&rank(b)).then_with(|| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()))
    });
    found
}

fn pick_default_entry(entries: &[String], preferred: Option<&str>) -> String {
    if let Some(preferred) = preferred {
        if entries.iter().any(|item| item == preferred) {
            return preferred.to_string();
        }
    }
    entries.first().cloned().unwrap_or_else(|| "index.html".into())
}

pub fn site_name(root: &Path) -> String {
    root.file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("网站")
        .to_string()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn make_access_code() -> String {
    let mut rng = rand::thread_rng();
    (0..4)
        .map(|_| ACCESS_ALPHABET[rng.gen_range(0..ACCESS_ALPHABET.len())] as char)
        .collect()
}

fn make_token() -> String {
    let mut rng = rand::thread_rng();
    (0..24)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}

async fn bind_port(ip: &str, start: u16) -> Result<(TcpListener, u16), String> {
    let mut port = start;
    let mut last_error = "无法监听端口".to_string();
    for _ in 0..20 {
        match TcpListener::bind((ip, port)).await {
            Ok(listener) => return Ok((listener, port)),
            Err(error) => last_error = error.to_string(),
        }
        port += 1;
    }
    Err(last_error)
}

fn router(context: HttpContext) -> Router {
    Router::new()
        .route("/__vibeshare/auth", post(handle_auth))
        .route("/__vibeshare/revision", get(handle_revision))
        .route("/__vibeshare/live.js", get(handle_live_script))
        .fallback(serve_path)
        .with_state(context)
}

async fn spawn_server(ip: &str, preferred_port: u16, context: HttpContext) -> Result<(RunningServer, u16), String> {
    let (listener, port) = bind_port(ip, preferred_port).await?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let app = router(context).into_make_service_with_connect_info::<SocketAddr>();
    let join = tokio::spawn(async move {
        let serve = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });
        let _ = serve.await;
    });
    Ok((
        RunningServer {
            port,
            shutdown: Some(shutdown_tx),
            join,
        },
        port,
    ))
}

fn stop_running(server: &mut Option<RunningServer>) {
    if let Some(mut running) = server.take() {
        if let Some(shutdown) = running.shutdown.take() {
            let _ = shutdown.send(());
        }
        running.join.abort();
    }
}

fn stop_share_locked(inner: &mut InnerState) {
    if let Some(session) = inner.share.take() {
        let mut server = Some(session.server);
        stop_running(&mut server);
    }
    // 代次前进：任何在飞的过期定时器醒来后都会发现自己已过期。
    inner.generation = inner.generation.wrapping_add(1);
}

pub fn status(state: &AppState) -> ShareStatus {
    let mut inner = state.inner.lock().expect("state");
    let network = current_network(inner.selected_ip.as_deref());
    let preview_url = inner.preview.as_ref().map(|server| format!("http://127.0.0.1:{}/", server.port));
    let proxy_port = inner.proxy_port;
    let mode = if proxy_port.is_some() { "proxy" } else { "static" }.to_string();
    if let Some(session) = inner.share.as_mut() {
        // 只在读状态时顺手清理：离开的设备不该继续算进"在线"数字里。
        let cutoff = now_ms().saturating_sub(DEVICE_TTL_MS);
        session.devices.retain(|_, device| device.last_seen >= cutoff);
    }
    if let Some(session) = &inner.share {
        let suffix = if session.code.is_empty() {
            String::new()
        } else {
            format!("?code={}", session.code)
        };
        ShareStatus {
            active: true,
            url: Some(format!("http://{}:{}/{}", session.lan_ip, session.server.port, suffix)),
            preview_url,
            access_code: if session.code.is_empty() {
                None
            } else {
                Some(session.code.clone())
            },
            expires_at: session.expires_at,
            connections: session.devices.len(),
            devices: sorted_devices(&session.devices),
            entry: inner.entry.clone(),
            root: inner.selected_root.as_ref().map(|path| path.display().to_string()),
            port: Some(session.server.port),
            mode,
            proxy_port,
            network,
        }
    } else {
        ShareStatus {
            active: false,
            url: None,
            preview_url,
            access_code: None,
            expires_at: None,
            connections: 0,
            devices: Vec::new(),
            entry: inner.entry.clone(),
            root: inner.selected_root.as_ref().map(|path| path.display().to_string()),
            port: inner.preview.as_ref().map(|server| server.port),
            mode,
            proxy_port,
            network,
        }
    }
}

pub async fn select_site(state: &AppState, root: PathBuf, preferred_entry: Option<String>) -> Result<SiteInfo, String> {
    let scan_root = root.clone();
    let scan_preferred = preferred_entry.clone();
    let (html_entries, entry, name) = tokio::task::spawn_blocking(move || -> Result<_, String> {
        if !scan_root.is_dir() {
            return Err("请选择网站目录".into());
        }
        let mut html_entries = if scan_preferred.is_some() {
            list_html_in_dir(&scan_root)
        } else {
            list_html_entries(&scan_root)
        };
        if let Some(preferred) = scan_preferred.as_deref() {
            let picked = scan_root.join(preferred);
            if !picked.is_file() {
                return Err("未找到所选 HTML 文件".into());
            }
            if !html_entries.iter().any(|item| item == preferred) {
                html_entries.insert(0, preferred.to_string());
            }
        }
        if html_entries.is_empty() {
            return Err("未找到 HTML 入口".into());
        }
        let entry = pick_default_entry(&html_entries, scan_preferred.as_deref());
        let name = site_name(&scan_root);
        Ok((html_entries, entry, name))
    })
    .await
    .map_err(|error| error.to_string())??;

    // 预览服务的 router 从共享状态里读站点根，换站点不需要重启它。
    // 原先每次都先 stop 再 spawn，而 `join.abort()` 并不保证端口立刻释放，
    // 于是端口会一路往上爬（4173 → 4174 → …），预览地址随之失效。
    let already_running = {
        let inner = state.inner.lock().expect("state");
        inner.preview.is_some()
    };
    let preview = if already_running {
        None
    } else {
        let context = HttpContext {
            state: state.clone(),
            require_code: false,
        };
        Some(spawn_server("127.0.0.1", DEFAULT_PORT, context).await?.0)
    };

    {
        let mut inner = state.inner.lock().expect("state");
        inner.selected_root = Some(root.clone());
        inner.entry = Some(entry.clone());
        if let Some(preview) = preview {
            inner.preview = Some(preview);
        }
    }

    state.rewatch(&root);

    Ok(SiteInfo {
        name,
        root: root.display().to_string(),
        entry,
        html_entries,
    })
}

pub fn set_entry(state: &AppState, entry: String) -> Result<SiteInfo, String> {
    let root = {
        let inner = state.inner.lock().expect("state");
        inner.selected_root.clone().ok_or("请先选择网站")?
    };
    if !root.join(&entry).is_file() {
        return Err("入口不存在".into());
    }
    let html_entries = list_html_in_dir(&root);
    let mut inner = state.inner.lock().expect("state");
    inner.entry = Some(entry.clone());
    Ok(SiteInfo {
        name: site_name(&root),
        root: root.display().to_string(),
        entry,
        html_entries,
    })
}

pub fn regenerate_code(state: &AppState) -> Result<ShareStatus, String> {
    {
        let mut inner = state.inner.lock().expect("state");
        let session = inner.share.as_mut().ok_or("未在共享")?;
        if session.code.is_empty() {
            return Err("访问码已关闭".into());
        }
        session.code = make_access_code();
        session.cookie = make_token();
    }
    Ok(status(state))
}

pub async fn start_share(state: &AppState, input: StartShareInput) -> Result<ShareStatus, String> {
    let (root, entry, preferred_port) = {
        let inner = state.inner.lock().expect("state");
        let root = inner.selected_root.clone().ok_or("请先选择网站")?;
        let entry = inner.entry.clone().ok_or("未找到入口")?;
        let port = inner.preview.as_ref().map(|server| server.port).unwrap_or(DEFAULT_PORT);
        (root, entry, port)
    };

    if input.network_ip.is_some() {
        let mut inner = state.inner.lock().expect("state");
        inner.selected_ip = input.network_ip.clone();
    }

    let network = {
        let inner = state.inner.lock().expect("state");
        current_network(inner.selected_ip.as_deref())
    };
    if !network.reachable {
        return Err("没有可用的局域网地址".into());
    }

    let protect = input.access_protection.unwrap_or(true);
    let code = if protect { make_access_code() } else { String::new() };
    let cookie = make_token();
    let expires_at = input.expiry_minutes.filter(|value| *value > 0).map(|minutes| now_ms() + minutes * 60 * 1000);

    let context = HttpContext {
        state: state.clone(),
        require_code: protect,
    };

    {
        let mut inner = state.inner.lock().expect("state");
        stop_share_locked(&mut inner);
    }

    // 预览监听在 127.0.0.1，共享监听在局域网地址，是两个不同的 socket 地址，
    // 所以可以沿用同一个端口号 —— 用户看到的两个链接端口一致，更好解释。
    let (server, _) = spawn_server(&network.ip, preferred_port, context).await?;

    let generation = {
        let mut inner = state.inner.lock().expect("state");
        inner.selected_root = Some(root);
        inner.entry = Some(entry);
        inner.share = Some(ShareSession {
            code,
            cookie,
            expires_at,
            lan_ip: network.ip.clone(),
            devices: HashMap::new(),
            blocked_ips: HashSet::new(),
            server,
        });
        inner.generation
    };

    if let Some(deadline) = expires_at {
        let state = state.clone();
        tokio::spawn(async move {
            let wait = deadline.saturating_sub(now_ms());
            tokio::time::sleep(Duration::from_millis(wait.max(1))).await;
            // 代次校验：用户可能已经手动停止并重开了共享，那是另一场会话，
            // 不该被这个旧定时器关掉。
            let mut inner = state.inner.lock().expect("state");
            if inner.generation == generation && inner.share.is_some() {
                stop_share_locked(&mut inner);
            }
        });
    }

    Ok(status(state))
}

pub fn stop_share(state: &AppState) -> ShareStatus {
    let mut inner = state.inner.lock().expect("state");
    stop_share_locked(&mut inner);
    drop(inner);
    status(state)
}

/// 仅收回某台设备对当前分享的访问权。下一次开始分享时不再受影响。
pub fn block_device(state: &AppState, ip: String) -> Result<ShareStatus, String> {
    let mut inner = state.inner.lock().expect("state");
    let Some(session) = inner.share.as_mut() else {
        return Err("当前没有进行中的分享".into());
    };
    session.blocked_ips.insert(ip.clone());
    session.devices.remove(&ip);
    drop(inner);
    Ok(status(state))
}

/// 加入黑名单后，设备不能再访问本应用此后发起的任何分享。
pub fn blacklist_device(state: &AppState, ip: String) -> Result<ShareStatus, String> {
    let mut inner = state.inner.lock().expect("state");
    inner.blacklisted_ips.insert(ip.clone());
    if let Some(session) = inner.share.as_mut() {
        session.blocked_ips.insert(ip.clone());
        session.devices.remove(&ip);
    }
    let path = inner.blacklist_path.clone();
    let entries: Vec<_> = inner.blacklisted_ips.iter().cloned().collect();
    drop(inner);
    persist_blacklist(path, &entries)?;
    Ok(status(state))
}

/// 已拉黑设备的稳定标识。当前网络层只能可靠地识别到 IP，因此列表按 IP 展示。
pub fn blacklisted_devices(state: &AppState) -> Vec<String> {
    let inner = state.inner.lock().expect("state");
    let mut entries: Vec<_> = inner.blacklisted_ips.iter().cloned().collect();
    entries.sort();
    entries
}

pub fn remove_blacklisted_device(state: &AppState, ip: String) -> Result<Vec<String>, String> {
    let mut inner = state.inner.lock().expect("state");
    inner.blacklisted_ips.remove(&ip);
    let path = inner.blacklist_path.clone();
    let mut entries: Vec<_> = inner.blacklisted_ips.iter().cloned().collect();
    entries.sort();
    drop(inner);
    persist_blacklist(path, &entries)?;
    Ok(entries)
}

fn persist_blacklist(path: Option<PathBuf>, entries: &[String]) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    let directory = path.parent().ok_or("无法保存设备黑名单")?;
    std::fs::create_dir_all(directory).map_err(|error| format!("无法保存设备黑名单：{error}"))?;
    let bytes = serde_json::to_vec_pretty(entries).map_err(|error| format!("无法保存设备黑名单：{error}"))?;
    std::fs::write(path, bytes).map_err(|error| format!("无法保存设备黑名单：{error}"))
}


/// 版本号端点。客户端脚本轮询它，值一变就刷新。
///
/// 这里是纯内存读取，比原先每次请求同步遍历整个目录便宜好几个数量级。
async fn handle_revision(State(context): State<HttpContext>) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(context.state.revision().to_string()))
        .unwrap()
}

/// 热更新脚本。
///
/// 关键决定：走**外部脚本**而不是内联 `<script>`。用户页面里只要有
/// `Content-Security-Policy: script-src 'self'`（AI 生成的页面和框架产物
/// 经常自带），内联脚本会被直接拦掉，热更新就静默失效 —— 桌面预览此前不
/// 刷新，最可能就是这个原因。同源的 `/__vibeshare/live.js` 能通过绝大多数
/// 现实中的 CSP。
const LIVE_SCRIPT: &str = r#"(() => {
  const url = "/__vibeshare/revision";
  let known = null;
  let failures = 0;
  const tick = async () => {
    try {
      const response = await fetch(url, { cache: "no-store" });
      if (!response.ok) throw new Error(String(response.status));
      const next = (await response.text()).trim();
      failures = 0;
      if (known === null) {
        known = next;
      } else if (next && next !== known) {
        location.reload();
      }
    } catch {
      // 共享停止后端点会消失，静默退避，不要在控制台刷屏。
      failures += 1;
      if (failures > 20) return;
    }
    setTimeout(tick, failures > 3 ? 3000 : 700);
  };
  tick();
})();
"#;

async fn handle_live_script() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(LIVE_SCRIPT))
        .unwrap()
}

const LIVE_TAG: &str = r#"<script src="/__vibeshare/live.js" id="__vibeshare_live" defer></script>"#;

/// 给 HTML 挂上热更新脚本。
///
/// 不再区分 loopback：手机访问者和桌面预览用的是同一套机制，这也是
/// "手机没有热更新"的直接修复。
fn inject_live_reload(bytes: Vec<u8>, mime: &str) -> Vec<u8> {
    if !mime.contains("html") {
        return bytes;
    }
    let Ok(mut html) = String::from_utf8(bytes.clone()) else {
        return bytes;
    };
    if html.contains("__vibeshare_live") {
        return bytes;
    }
    let lower = html.to_ascii_lowercase();
    if let Some(index) = lower.rfind("</body>") {
        html.insert_str(index, LIVE_TAG);
    } else {
        html.push_str(LIVE_TAG);
    }
    html.into_bytes()
}


fn sorted_devices(devices: &HashMap<String, ConnectedDevice>) -> Vec<ConnectedDevice> {
    let mut list: Vec<_> = devices.values().cloned().collect();
    list.sort_by(|a, b| b.last_seen.cmp(&a.last_seen).then_with(|| a.ip.cmp(&b.ip)));
    list
}

fn name_from_user_agent(ua: &str) -> String {
    let lower = ua.to_ascii_lowercase();
    if lower.contains("iphone") {
        "iPhone".into()
    } else if lower.contains("ipad") {
        "iPad".into()
    } else if lower.contains("android") {
        "Android 设备".into()
    } else if lower.contains("macintosh") || lower.contains("mac os") {
        "Mac".into()
    } else if lower.contains("windows") {
        "Windows 电脑".into()
    } else if lower.contains("linux") {
        "Linux 设备".into()
    } else if ua.trim().is_empty() {
        "未知设备".into()
    } else {
        "浏览器".into()
    }
}

/// 设备形态，只用来选图标。`Android` 平板在 UA 里通常不带 `mobile`。
fn kind_from_user_agent(ua: &str) -> &'static str {
    let lower = ua.to_ascii_lowercase();
    if lower.contains("ipad") || (lower.contains("android") && !lower.contains("mobile")) {
        "tablet"
    } else if lower.contains("iphone") || lower.contains("android") || lower.contains("mobile") {
        "phone"
    } else if lower.contains("macintosh") || lower.contains("mac os") || lower.contains("windows") || lower.contains("linux") {
        "desktop"
    } else {
        "unknown"
    }
}


fn reverse_lookup_name(ip: &str) -> Option<String> {
    let output = std::process::Command::new("dscacheutil")
        .args(["-q", "host", "-a", "ip_address", ip])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if let Some(name) = line.strip_prefix("name:") {
            let name = name.trim().trim_end_matches('.');
            if !name.is_empty() && name != ip {
                return Some(name.split('.').next().unwrap_or(name).to_string());
            }
        }
    }
    None
}

fn enrich_device_name(state: &AppState, ip: String) {
    let Some(name) = reverse_lookup_name(&ip) else {
        return;
    };
    let Ok(mut inner) = state.inner.lock() else {
        return;
    };
    if let Some(session) = inner.share.as_mut() {
        if let Some(device) = session.devices.get_mut(&ip) {
            device.name = name;
        }
    }
}

/// 记录一次访问。返回值表示这是不是一台新出现的设备 —— 只有新设备才值得
/// 去 fork `dscacheutil` 反查主机名，否则每个请求都会开一个进程。
fn touch_device(devices: &mut HashMap<String, ConnectedDevice>, ip: &str, ua: &str) -> bool {
    if ip == "127.0.0.1" || ip == "::1" || ip.starts_with("::ffff:127.") {
        return false;
    }
    let now = now_ms();
    if let Some(existing) = devices.get_mut(ip) {
        existing.last_seen = now;
        let next = name_from_user_agent(ua);
        if existing.name == "未知设备" && next != "未知设备" {
            existing.name = next;
            existing.kind = kind_from_user_agent(ua).to_string();
        }
        return false;
    }
    devices.insert(
        ip.to_string(),
        ConnectedDevice {
            ip: ip.to_string(),
            name: name_from_user_agent(ua),
            kind: kind_from_user_agent(ua).to_string(),
            last_seen: now,
        },
    );
    true
}

fn is_loopback(addr: &SocketAddr) -> bool {
    match addr.ip() {
        std::net::IpAddr::V4(ip) => ip.is_loopback(),
        std::net::IpAddr::V6(ip) => ip.is_loopback(),
    }
}

fn parse_cookies(headers: &HeaderMap) -> HashMap<String, String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .split(';')
        .filter_map(|part| {
            let (key, value) = part.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn access_page(message: &str) -> Html<String> {
    Html(format!(
        r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>输入访问码</title>
<style>body{{margin:0;min-height:100vh;display:grid;place-items:center;background:#f4f6f8;font-family:-apple-system,sans-serif;color:#263548}}main{{width:min(360px,calc(100% - 40px));padding:28px;background:#fff;border:1px solid #dfe5ec;border-radius:12px}}input,button{{width:100%;height:42px;box-sizing:border-box}}input{{padding:0 12px;letter-spacing:.16em;text-transform:uppercase}}button{{margin-top:14px;border:0;border-radius:7px;background:#4776e6;color:#fff}}small{{color:#b44f4a}}</style></head>
<body><main><h1>输入访问码</h1><form method="post" action="/__vibeshare/auth"><label>访问码</label><input name="code" maxlength="8" autofocus required>{message}<button type="submit">打开</button></form></main></body></html>"#,
        message = if message.is_empty() {
            String::new()
        } else {
            format!("<small>{message}</small>")
        }
    ))
}

/// 把请求路径解析成共享根下的真实文件。
///
/// 必须先 percent-decode：浏览器会把空格编码成 `%20`、中文编码成 `%E4%B8%AD`，
/// 不解码就永远找不到这些文件 —— 这是"文件名带空格就 404"的根因。
/// 解码后再做 `..` 与前缀校验，顺序不能反（否则 `%2e%2e` 能绕过检查）。
fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, StatusCode> {
    let decoded = percent_decode_str(relative).decode_utf8_lossy().to_string();
    let cleaned = decoded.trim_start_matches('/');
    if cleaned.contains('\0') || cleaned.split('/').any(|part| part == "..") {
        return Err(StatusCode::FORBIDDEN);
    }
    let candidate = root.join(cleaned);
    let root = root.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
    let resolved = candidate.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
    if !resolved.starts_with(&root) {
        return Err(StatusCode::FORBIDDEN);
    }
    if !resolved.is_file() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(resolved)
}

/// 请求看起来是要一个页面（而不是资源文件）吗？
///
/// SPA（Vue Router / React Router 的 history 模式）会请求 `/about` 这样磁盘上
/// 并不存在的路径。对这类请求回落到入口 HTML，路由器就能自己接手；对
/// `/app.js` 这种带扩展名的请求必须老实返回 404，否则前端会拿到一坨 HTML
/// 当 JS 执行，报错完全不指向真正原因。
fn wants_document(path: &str, headers: &HeaderMap) -> bool {
    let last = path.rsplit('/').next().unwrap_or("");
    if last.contains('.') {
        return false;
    }
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("text/html"))
        .unwrap_or(false)
}

async fn serve_path(
    State(context): State<HttpContext>,
    axum::extract::ConnectInfo(addr): axum::extract::ConnectInfo<SocketAddr>,
    Query(query): Query<HashMap<String, String>>,
    request: Request<Body>,
) -> Response {
    let headers = request.headers().clone();
    let path = request.uri().path().to_string();
    let (root, entry, proxy_port, need_auth, expected_code, cookie_token, is_new_device) = {
        let mut inner = context.state.inner.lock().expect("state");
        let ip = addr.ip().to_string();
        if inner.blacklisted_ips.contains(&ip)
            || inner
                .share
                .as_ref()
                .is_some_and(|session| session.blocked_ips.contains(&ip))
        {
            return (StatusCode::FORBIDDEN, "此设备已被禁止访问本次分享").into_response();
        }
        let proxy_port = inner.proxy_port;
        // 代理模式下没有站点根也照样能服务，站点根只在静态模式必需。
        let root = inner.selected_root.clone();
        if root.is_none() && proxy_port.is_none() {
            return (StatusCode::GONE, "共享已停止").into_response();
        }
        let entry = inner.entry.clone().unwrap_or_else(|| "index.html".into());
        let mut is_new_device = false;
        let (need_auth, expected_code, cookie_token) = if let Some(session) = inner.share.as_mut() {
            let ua = headers
                .get(header::USER_AGENT)
                .and_then(|value| value.to_str().ok())
                .unwrap_or("");
            is_new_device = touch_device(&mut session.devices, &ip, ua);
            if let Some(deadline) = session.expires_at {
                if now_ms() >= deadline {
                    return (StatusCode::GONE, "共享已停止").into_response();
                }
            }
            (context.require_code, session.code.clone(), session.cookie.clone())
        } else {
            (false, String::new(), String::new())
        };
        (root, entry, proxy_port, need_auth, expected_code, cookie_token, is_new_device)
    };

    if is_new_device && !is_loopback(&addr) {
        let enrich_state = context.state.clone();
        let enrich_ip = addr.ip().to_string();
        tokio::task::spawn_blocking(move || enrich_device_name(&enrich_state, enrich_ip));
    }

    if need_auth && !is_loopback(&addr) {
        let cookies = parse_cookies(&headers);
        let cookie_ok = cookies.get("vibeshare_access").map(String::as_str) == Some(cookie_token.as_str());
        let query_ok = query.get("code").map(String::as_str) == Some(expected_code.as_str()) && !expected_code.is_empty();
        if query_ok && !cookie_ok {
            let mut response = Redirect::temporary(&path).into_response();
            response.headers_mut().insert(
                header::SET_COOKIE,
                format!("vibeshare_access={cookie_token}; Path=/; HttpOnly; SameSite=Lax")
                    .parse()
                    .unwrap(),
            );
            return response;
        }
        if !cookie_ok {
            let message = if query.contains_key("code") {
                "访问码不正确"
            } else {
                ""
            };
            return (StatusCode::UNAUTHORIZED, access_page(message)).into_response();
        }
    }

    // 代理模式：鉴权通过后原样转发，包括 WebSocket 升级。框架自带的 HMR
    // 走的就是这条通道，所以代理模式不需要（也不应该）再注入我们的脚本。
    if let Some(port) = proxy_port {
        return proxy::forward(port, request).await;
    }

    let Some(root) = root else {
        return (StatusCode::GONE, "共享已停止").into_response();
    };

    let relative = if path == "/" {
        entry.clone()
    } else {
        path.trim_start_matches('/').to_string()
    };
    let file = match safe_join(&root, &relative) {
        Ok(file) => Some(file),
        Err(status) => {
            if status == StatusCode::NOT_FOUND && wants_document(&path, &headers) {
                // SPA history 回落到入口 HTML。
                safe_join(&root, &entry).ok()
            } else {
                return status.into_response();
            }
        }
    };
    let Some(file) = file else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let bytes = match tokio::fs::read(&file).await {
        Ok(bytes) => bytes,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };
    let mime = mime_guess::from_path(&file).first_or_octet_stream();
    let body = inject_live_reload(bytes, mime.essence_str());
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime.essence_str())
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::PRAGMA, "no-cache")
        .body(Body::from(body))
        .unwrap()
}

#[derive(Deserialize)]
struct AuthForm {
    code: String,
}

/// 确保预览服务在跑。代理模式下用户可能根本没有选目录，但依然需要一个
/// 本机地址来预览。
async fn ensure_preview(state: &AppState) -> Result<u16, String> {
    if let Some(port) = state.inner.lock().expect("state").preview.as_ref().map(|server| server.port) {
        return Ok(port);
    }
    let context = HttpContext {
        state: state.clone(),
        require_code: false,
    };
    let (preview, port) = spawn_server("127.0.0.1", DEFAULT_PORT, context).await?;
    state.inner.lock().expect("state").preview = Some(preview);
    Ok(port)
}

/// VibeShare 自己占用的端口。代理到这些端口会造成请求无限自环。
fn own_ports(inner: &InnerState) -> Vec<u16> {
    let mut ports = Vec::new();
    if let Some(server) = &inner.preview {
        ports.push(server.port);
    }
    if let Some(session) = &inner.share {
        ports.push(session.server.port);
    }
    ports
}

/// 探测本机在跑的开发服务器。
pub async fn detect_dev_servers(state: &AppState) -> Vec<DevServer> {
    let busy = {
        let inner = state.inner.lock().expect("state");
        own_ports(&inner)
    };
    proxy::detect(&busy).await
}

/// 切换到代理模式，把所有请求转发给 `127.0.0.1:port`。
///
/// 这一步让"支持动态网页"成立：SSR、后端 API、框架自带的 HMR 全部由用户的
/// dev server 自己处理，VibeShare 不需要理解任何具体框架。
pub async fn use_dev_server(state: &AppState, port: u16) -> Result<ShareStatus, String> {
    {
        let inner = state.inner.lock().expect("state");
        if own_ports(&inner).contains(&port) {
            return Err("这是 VibeShare 自己的端口，转发会造成死循环".into());
        }
    }
    if tokio::time::timeout(
        Duration::from_millis(500),
        tokio::net::TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .map_err(|_| format!("连接 127.0.0.1:{port} 超时"))?
    .is_err()
    {
        return Err(format!("127.0.0.1:{port} 没有在监听"));
    }
    ensure_preview(state).await?;
    {
        let mut inner = state.inner.lock().expect("state");
        inner.proxy_port = Some(port);
        // 代理模式由 dev server 自己负责热更新，我们的文件监听没有意义。
        inner.watcher = None;
    }
    Ok(status(state))
}

/// 回到静态直出模式。
pub fn use_static_mode(state: &AppState) -> ShareStatus {
    let root = {
        let mut inner = state.inner.lock().expect("state");
        inner.proxy_port = None;
        inner.selected_root.clone()
    };
    if let Some(root) = root {
        state.rewatch(&root);
    }
    status(state)
}

/// 切换共享用的网卡。正在共享时需要换地址重开监听，访问码与到期时间保留 ——
/// 用户换的是网络，不是会话。
pub async fn set_network_ip(state: &AppState, ip: Option<String>) -> Result<ShareStatus, String> {
    let previous = {
        let mut inner = state.inner.lock().expect("state");
        inner.selected_ip = ip.clone();
        inner.share.as_ref().map(|session| {
            (
                session.code.clone(),
                session.cookie.clone(),
                session.expires_at,
                session.server.port,
                session.lan_ip.clone(),
            )
        })
    };
    // 没在共享时只记住选择，下次开始共享自然会用上。
    let Some((code, cookie, expires_at, port, current_ip)) = previous else {
        return Ok(status(state));
    };

    let network = current_network(ip.as_deref());
    if !network.reachable {
        return Err("这个网卡没有可用地址".into());
    }
    if network.ip == current_ip {
        return Ok(status(state));
    }

    let context = HttpContext {
        state: state.clone(),
        require_code: !code.is_empty(),
    };
    {
        let mut inner = state.inner.lock().expect("state");
        stop_share_locked(&mut inner);
    }
    let (server, _) = spawn_server(&network.ip, port, context).await?;
    {
        let mut inner = state.inner.lock().expect("state");
        inner.share = Some(ShareSession {
            code,
            cookie,
            expires_at,
            lan_ip: network.ip.clone(),
            devices: HashMap::new(),
            blocked_ips: HashSet::new(),
            server,
        });
    }
    Ok(status(state))
}

/// 扫描入口页面的资源引用。代理模式下页面由 dev server 生成，磁盘上没有可
/// 扫描的产物，直接返回空报告。
pub async fn diagnose_assets(state: &AppState) -> AssetReport {
    let target = {
        let inner = state.inner.lock().expect("state");
        if inner.proxy_port.is_some() {
            None
        } else {
            match (inner.selected_root.clone(), inner.entry.clone()) {
                (Some(root), Some(entry)) => Some((root, entry)),
                _ => None,
            }
        }
    };
    let Some((root, entry)) = target else {
        return AssetReport::default();
    };
    tokio::task::spawn_blocking(move || diagnose::scan(&root, &entry))
        .await
        .unwrap_or_default()
}

async fn handle_auth(
    State(context): State<HttpContext>,
    Form(form): Form<AuthForm>,
) -> Response {
    let expected = {
        let inner = context.state.inner.lock().expect("state");
        inner.share.as_ref().map(|session| (session.code.clone(), session.cookie.clone()))
    };
    let Some((code, cookie)) = expected else {
        return (StatusCode::GONE, "共享已停止").into_response();
    };
    if form.code.trim().to_ascii_uppercase() != code {
        return (StatusCode::UNAUTHORIZED, access_page("访问码不正确")).into_response();
    }
    let mut response = Redirect::temporary("/").into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        format!("vibeshare_access={cookie}; Path=/; HttpOnly; SameSite=Lax")
            .parse()
            .unwrap(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("vibeshare-test-{name}"));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn safe_join_decodes_percent_escapes() {
        let root = temp_root("percent");
        std::fs::write(root.join("我的 页面.html"), b"<html></html>").expect("write");
        let resolved = safe_join(&root, "/%E6%88%91%E7%9A%84%20%E9%A1%B5%E9%9D%A2.html");
        assert!(resolved.is_ok(), "percent-encoded path should resolve");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn safe_join_rejects_traversal_including_encoded() {
        let root = temp_root("traversal");
        std::fs::write(root.join("index.html"), b"<html></html>").expect("write");
        assert_eq!(safe_join(&root, "../secret.txt"), Err(StatusCode::FORBIDDEN));
        // 先解码再校验，所以编码过的 `..` 同样被拦下。
        assert_eq!(safe_join(&root, "%2e%2e/secret.txt"), Err(StatusCode::FORBIDDEN));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn safe_join_rejects_directories() {
        let root = temp_root("dir");
        std::fs::create_dir_all(root.join("assets")).expect("mkdir");
        assert_eq!(safe_join(&root, "assets"), Err(StatusCode::NOT_FOUND));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn finds_entry_in_build_output_dir() {
        let root = temp_root("dist");
        std::fs::create_dir_all(root.join("dist")).expect("mkdir");
        std::fs::write(root.join("dist/index.html"), b"<html></html>").expect("write");
        // `dist` 不在忽略清单里，所以入口能被扫到。
        let entries = list_html_entries(&root);
        assert_eq!(entries, vec!["dist/index.html".to_string()]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn prefers_root_index_over_nested() {
        let root = temp_root("rank");
        std::fs::create_dir_all(root.join("docs")).expect("mkdir");
        std::fs::write(root.join("index.html"), b"<html></html>").expect("write");
        std::fs::write(root.join("docs/index.html"), b"<html></html>").expect("write");
        let entries = list_html_entries(&root);
        assert_eq!(entries.first().map(String::as_str), Some("index.html"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn injects_external_script_once() {
        let html = b"<html><body>hi</body></html>".to_vec();
        let first = inject_live_reload(html, "text/html");
        let text = String::from_utf8(first.clone()).expect("utf8");
        assert!(text.contains("/__vibeshare/live.js"));
        assert_eq!(text.matches("__vibeshare_live").count(), 1);
        // 重复注入必须是幂等的。
        let second = inject_live_reload(first.clone(), "text/html");
        assert_eq!(first, second);
    }

    #[test]
    fn does_not_inject_into_non_html() {
        let js = b"console.log(1)".to_vec();
        assert_eq!(inject_live_reload(js.clone(), "text/javascript"), js);
    }

    #[test]
    fn injects_even_without_body_tag() {
        let html = b"<h1>fragment</h1>".to_vec();
        let text = String::from_utf8(inject_live_reload(html, "text/html")).expect("utf8");
        assert!(text.contains("/__vibeshare/live.js"));
    }

    fn accept_html() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, "text/html,application/xhtml+xml".parse().unwrap());
        headers
    }

    #[test]
    fn spa_fallback_only_for_extensionless_documents() {
        assert!(wants_document("/about", &accept_html()));
        assert!(wants_document("/users/42", &accept_html()));
        // 带扩展名的资源必须老实 404，否则前端会把 HTML 当 JS 执行。
        assert!(!wants_document("/app.js", &accept_html()));
        assert!(!wants_document("/img/logo.png", &accept_html()));
        // fetch / XHR 不带 text/html，也不该拿到入口页。
        assert!(!wants_document("/api/users", &HeaderMap::new()));
    }

    #[test]
    fn classifies_device_kinds() {
        assert_eq!(kind_from_user_agent("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0)"), "phone");
        assert_eq!(kind_from_user_agent("Mozilla/5.0 (iPad; CPU OS 17_0)"), "tablet");
        assert_eq!(kind_from_user_agent("Mozilla/5.0 (Linux; Android 14; Pixel) Mobile"), "phone");
        assert_eq!(kind_from_user_agent("Mozilla/5.0 (Linux; Android 14; Tab)"), "tablet");
        assert_eq!(kind_from_user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X)"), "desktop");
        assert_eq!(kind_from_user_agent(""), "unknown");
    }

    #[test]
    fn skips_loopback_devices_and_reports_new_ones() {
        let mut devices = HashMap::new();
        assert!(!touch_device(&mut devices, "127.0.0.1", "curl"));
        assert!(devices.is_empty());
        assert!(touch_device(&mut devices, "192.168.1.20", "Mozilla/5.0 (iPhone)"));
        // 第二次访问不是新设备，所以不会再 fork 反查进程。
        assert!(!touch_device(&mut devices, "192.168.1.20", "Mozilla/5.0 (iPhone)"));
        assert_eq!(devices.len(), 1);
        assert_eq!(devices["192.168.1.20"].kind, "phone");
    }

    #[test]
    fn sorts_devices_by_recency() {
        let mut devices = HashMap::new();
        devices.insert(
            "10.0.0.2".to_string(),
            ConnectedDevice {
                ip: "10.0.0.2".into(),
                name: "旧".into(),
                kind: "phone".into(),
                last_seen: 100,
            },
        );
        devices.insert(
            "10.0.0.3".to_string(),
            ConnectedDevice {
                ip: "10.0.0.3".into(),
                name: "新".into(),
                kind: "phone".into(),
                last_seen: 200,
            },
        );
        let sorted = sorted_devices(&devices);
        assert_eq!(sorted[0].ip, "10.0.0.3");
    }

    #[test]
    fn access_code_uses_unambiguous_alphabet() {
        for _ in 0..200 {
            let code = make_access_code();
            assert_eq!(code.len(), 4);
            // 排除 0/O/1/I，念给别人听不会错。
            assert!(!code.contains(['0', 'O', '1', 'I']));
        }
    }
}
