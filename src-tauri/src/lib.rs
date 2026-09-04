mod diagnose;
mod network;
mod proxy;
mod qr;
mod share;
mod watch;
#[cfg(target_os = "macos")]
mod macos;

use diagnose::AssetReport;
use proxy::DevServer;
use share::{
    detect_dev_servers, diagnose_assets, regenerate_code, select_site, set_entry, set_network_ip, start_share, status,
    stop_share, use_dev_server, use_static_mode, AppState, ShareStatus, SiteInfo, StartShareInput,
};
use std::path::PathBuf;
use tauri::{Manager, RunEvent};
use tauri_plugin_dialog::DialogExt;

fn apply_split_min_size(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    macos::apply_fullscreen_tile_limits(window);
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
    }
}


#[tauri::command]
fn network_info(state: tauri::State<AppState>) -> ShareStatus {
    status(&state)
}

async fn pick_path(
    app: &tauri::AppHandle,
    title: &str,
    html_only: bool,
) -> Result<PathBuf, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let mut dialog = app.dialog().file().set_title(title);
    if html_only {
        dialog = dialog.add_filter("HTML", &["html", "htm"]);
        dialog.pick_file(move |file| {
            let _ = tx.send(file);
        });
    } else {
        dialog.pick_folder(move |file| {
            let _ = tx.send(file);
        });
    }
    let Some(file) = rx.await.map_err(|_| "已取消".to_string())? else {
        return Err("已取消".into());
    };
    Ok(PathBuf::from(file.to_string()))
}

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<SiteInfo, String> {
    let path = pick_path(&app, "选择网站目录", false).await?;
    select_site(&state, path, None).await
}

#[tauri::command]
async fn pick_html_file(app: tauri::AppHandle, state: tauri::State<'_, AppState>) -> Result<SiteInfo, String> {
    let path = pick_path(&app, "选择 HTML 文件", true).await?;
    let parent = path.parent().ok_or("无法解析目录")?.to_path_buf();
    let entry = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("index.html")
        .to_string();
    select_site(&state, parent, Some(entry)).await
}

#[tauri::command]
async fn open_site(
    state: tauri::State<'_, AppState>,
    path: String,
    entry: Option<String>,
) -> Result<SiteInfo, String> {
    select_site(&state, PathBuf::from(path), entry).await
}

#[tauri::command]
fn set_site_entry(state: tauri::State<AppState>, entry: String) -> Result<SiteInfo, String> {
    set_entry(&state, entry)
}

#[tauri::command]
fn share_status(state: tauri::State<AppState>) -> ShareStatus {
    status(&state)
}

#[tauri::command]
async fn share_start(state: tauri::State<'_, AppState>, input: StartShareInput) -> Result<ShareStatus, String> {
    start_share(&state, input).await
}

#[tauri::command]
fn share_stop(state: tauri::State<AppState>) -> ShareStatus {
    stop_share(&state)
}

#[tauri::command]
fn share_regenerate_code(state: tauri::State<AppState>) -> Result<ShareStatus, String> {
    regenerate_code(&state)
}

/// 列出本机在跑的开发服务器，供用户选择要转发哪一个。
#[tauri::command]
async fn dev_servers(state: tauri::State<'_, AppState>) -> Result<Vec<DevServer>, String> {
    Ok(detect_dev_servers(&state).await)
}

#[tauri::command]
async fn use_dev_server_port(state: tauri::State<'_, AppState>, port: u16) -> Result<ShareStatus, String> {
    use_dev_server(&state, port).await
}

#[tauri::command]
fn use_static_files(state: tauri::State<AppState>) -> ShareStatus {
    use_static_mode(&state)
}

#[tauri::command]
async fn select_network(state: tauri::State<'_, AppState>, ip: Option<String>) -> Result<ShareStatus, String> {
    set_network_ip(&state, ip).await
}

#[tauri::command]
async fn asset_report(state: tauri::State<'_, AppState>) -> Result<AssetReport, String> {
    Ok(diagnose_assets(&state).await)
}

#[tauri::command]
fn qr_png(text: String) -> Result<String, String> {
    qr::png_data_url(&text)
}

#[tauri::command]
fn share_block_device(state: tauri::State<AppState>, ip: String) -> Result<ShareStatus, String> {
    share::block_device(&state, ip)
}

#[tauri::command]
fn share_blacklist_device(state: tauri::State<AppState>, ip: String) -> Result<ShareStatus, String> {
    share::blacklist_device(&state, ip)
}

#[tauri::command]
fn share_blacklisted_devices(state: tauri::State<AppState>) -> Vec<String> {
    share::blacklisted_devices(&state)
}

#[tauri::command]
fn share_remove_blacklisted_device(state: tauri::State<AppState>, ip: String) -> Result<Vec<String>, String> {
    share::remove_blacklisted_device(&state, ip)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            network_info,
            pick_folder,
            pick_html_file,
            open_site,
            set_site_entry,
            share_status,
            share_start,
            share_stop,
            share_regenerate_code,
            dev_servers,
            use_dev_server_port,
            use_static_files,
            select_network,
            asset_report,
            qr_png,
            share_block_device,
            share_blacklist_device,
            share_blacklisted_devices,
            share_remove_blacklisted_device
        ])
        .setup(|app| {
            if let Ok(directory) = app.path().app_data_dir() {
                app.state::<AppState>().configure_blacklist_storage(directory);
            }
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("");
                #[cfg(target_os = "macos")]
                macos::install(&window);
                #[cfg(not(target_os = "macos"))]
                apply_split_min_size(&window);
            }
            // 热更新不再需要后台轮询：站点监听器由 select_site 安装，
            // 页面自己轮询 /__vibeshare/revision 决定何时刷新。
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building VibeShare")
        .run(|app, event| {
            if let RunEvent::Ready = event {
                if let Some(window) = app.get_webview_window("main") {
                    apply_split_min_size(&window);
                }
            }
        });
}
