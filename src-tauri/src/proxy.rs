//! 反向代理本机开发服务器。
//!
//! 静态模式只能直出磁盘文件，能力上限是"浏览器不依赖本机后端就能完成的事"。
//! 代理模式把请求原样转发到 `127.0.0.1:PORT`，于是 SSR、后端 API、框架自带的
//! HMR 全部自动可用 —— VibeShare 不需要理解任何具体框架。
//!
//! WebSocket 升级会被透传，这是 Vite / Next.js 热更新能穿过来的关键。

use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Request, Response, StatusCode};
use axum::response::IntoResponse;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use std::time::Duration;
use tokio::net::TcpStream;

/// 常见开发服务器端口。探测时会跳过 VibeShare 自己占用的端口。
pub const CANDIDATE_PORTS: [u16; 14] = [
    5173, 5174, 3000, 3001, 4200, 8080, 8000, 4321, 1420, 5500, 9000, 3333, 8788, 7357,
];

/// 逐跳首部，普通请求转发时必须剔除，否则连接语义会串掉。
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];

fn is_hop_by_hop(name: &str) -> bool {
    HOP_BY_HOP.iter().any(|item| item.eq_ignore_ascii_case(name))
}

#[derive(Debug, Clone, Serialize)]
pub struct DevServer {
    pub port: u16,
    pub title: String,
}

fn upstream_host(port: u16) -> HeaderValue {
    HeaderValue::from_str(&format!("127.0.0.1:{port}")).unwrap_or(HeaderValue::from_static("127.0.0.1"))
}

fn is_websocket(headers: &HeaderMap) -> bool {
    headers
        .get(header::UPGRADE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("websocket"))
        .unwrap_or(false)
}

/// 上游连不上时给出可读的提示页，而不是裸 502。
fn unreachable(port: u16, reason: &str) -> Response<Body> {
    let html = format!(
        r#"<!doctype html><html lang="zh-CN"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1"><title>开发服务未响应</title>
<style>body{{margin:0;min-height:100vh;display:grid;place-items:center;background:#f4f6f8;
font-family:-apple-system,sans-serif;color:#263548}}main{{width:min(420px,calc(100% - 40px));padding:28px;
background:#fff;border:1px solid #dfe5ec;border-radius:12px}}code{{background:#f4f6f8;padding:2px 6px;
border-radius:4px}}p{{line-height:1.6;color:#5a6b7f}}</style></head>
<body><main><h1>开发服务未响应</h1>
<p>VibeShare 正在把请求转发到 <code>127.0.0.1:{port}</code>，但连接失败。</p>
<p>请确认开发服务仍在运行，端口没有变化。</p>
<p style="color:#b44f4a">{reason}</p></main></body></html>"#
    );
    (StatusCode::BAD_GATEWAY, axum::response::Html(html)).into_response()
}

/// 转发一个普通 HTTP 请求。
pub async fn forward(port: u16, request: Request<Body>) -> Response<Body> {
    if is_websocket(request.headers()) {
        return tunnel(port, request).await;
    }

    let (parts, body) = request.into_parts();
    let target = parts
        .uri
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "/".into());

    let stream = match TcpStream::connect(("127.0.0.1", port)).await {
        Ok(stream) => stream,
        Err(error) => return unreachable(port, &error.to_string()),
    };
    let (mut sender, connection) = match hyper::client::conn::http1::handshake(TokioIo::new(stream)).await {
        Ok(pair) => pair,
        Err(error) => return unreachable(port, &error.to_string()),
    };
    tokio::spawn(async move {
        let _ = connection.await;
    });

    let mut outgoing = Request::builder().method(parts.method.clone()).uri(target);
    if let Some(headers) = outgoing.headers_mut() {
        for (name, value) in parts.headers.iter() {
            if is_hop_by_hop(name.as_str()) || name == header::HOST {
                continue;
            }
            headers.append(name.clone(), value.clone());
        }
        headers.insert(header::HOST, upstream_host(port));
    }
    let outgoing = match outgoing.body(body) {
        Ok(request) => request,
        Err(error) => return unreachable(port, &error.to_string()),
    };

    let upstream = match sender.send_request(outgoing).await {
        Ok(response) => response,
        Err(error) => return unreachable(port, &error.to_string()),
    };

    let (upstream_parts, upstream_body) = upstream.into_parts();
    let mut response = Response::builder().status(upstream_parts.status);
    if let Some(headers) = response.headers_mut() {
        for (name, value) in upstream_parts.headers.iter() {
            if is_hop_by_hop(name.as_str()) {
                continue;
            }
            headers.append(name.clone(), value.clone());
        }
    }
    response
        .body(Body::new(upstream_body))
        .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
}

/// 透传 WebSocket 升级：先把握手转发给上游，拿到 101 之后把两端的
/// 升级后连接对接起来做双向拷贝。框架的 HMR 通道就是走这条路。
async fn tunnel(port: u16, mut request: Request<Body>) -> Response<Body> {
    let downstream_upgrade = hyper::upgrade::on(&mut request);
    let (parts, _) = request.into_parts();
    let target = parts
        .uri
        .path_and_query()
        .map(|value| value.as_str().to_string())
        .unwrap_or_else(|| "/".into());

    let stream = match TcpStream::connect(("127.0.0.1", port)).await {
        Ok(stream) => stream,
        Err(error) => return unreachable(port, &error.to_string()),
    };
    let (mut sender, connection) = match hyper::client::conn::http1::handshake(TokioIo::new(stream)).await {
        Ok(pair) => pair,
        Err(error) => return unreachable(port, &error.to_string()),
    };
    let driver = tokio::spawn(async move {
        let _ = connection.with_upgrades().await;
    });

    // 握手请求必须保留 Connection / Upgrade / Sec-WebSocket-* 首部，这里只换 Host。
    let mut outgoing = Request::builder().method(parts.method.clone()).uri(target);
    if let Some(headers) = outgoing.headers_mut() {
        for (name, value) in parts.headers.iter() {
            if name == header::HOST {
                continue;
            }
            headers.append(name.clone(), value.clone());
        }
        headers.insert(header::HOST, upstream_host(port));
    }
    let outgoing = match outgoing.body(Body::empty()) {
        Ok(request) => request,
        Err(error) => {
            driver.abort();
            return unreachable(port, &error.to_string());
        }
    };

    let upstream = match sender.send_request(outgoing).await {
        Ok(response) => response,
        Err(error) => {
            driver.abort();
            return unreachable(port, &error.to_string());
        }
    };

    if upstream.status() != StatusCode::SWITCHING_PROTOCOLS {
        driver.abort();
        let (upstream_parts, upstream_body) = upstream.into_parts();
        let mut response = Response::builder().status(upstream_parts.status);
        if let Some(headers) = response.headers_mut() {
            for (name, value) in upstream_parts.headers.iter() {
                if is_hop_by_hop(name.as_str()) {
                    continue;
                }
                headers.append(name.clone(), value.clone());
            }
        }
        return response
            .body(Body::new(upstream_body))
            .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response());
    }

    let handshake_headers = upstream.headers().clone();
    let upstream_upgrade = hyper::upgrade::on(upstream);

    tokio::spawn(async move {
        if let Ok((downstream, upstream)) = tokio::try_join!(downstream_upgrade, upstream_upgrade) {
            let mut downstream = TokioIo::new(downstream);
            let mut upstream = TokioIo::new(upstream);
            let _ = tokio::io::copy_bidirectional(&mut downstream, &mut upstream).await;
        }
        driver.abort();
    });

    let mut response = Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);
    if let Some(headers) = response.headers_mut() {
        for (name, value) in handshake_headers.iter() {
            headers.append(name.clone(), value.clone());
        }
    }
    response
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
}

/// 从 HTML 里抓 `<title>`，用于让候选列表可读。
fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let open = lower[start..].find('>')? + start + 1;
    let end = lower[open..].find("</title>")? + open;
    let title = html[open..end].trim();
    if title.is_empty() {
        return None;
    }
    Some(title.chars().take(40).collect())
}

/// 向单个端口发一次 GET /，确认它是个 HTTP 服务并尽量拿到标题。
async fn probe(port: u16) -> Option<DevServer> {
    let connect = tokio::time::timeout(Duration::from_millis(200), TcpStream::connect(("127.0.0.1", port)));
    let stream = connect.await.ok()?.ok()?;
    let (mut sender, connection) = hyper::client::conn::http1::handshake(TokioIo::new(stream)).await.ok()?;
    let driver = tokio::spawn(async move {
        let _ = connection.await;
    });

    let request = Request::builder()
        .method("GET")
        .uri("/")
        .header(header::HOST, upstream_host(port))
        .header(header::USER_AGENT, "VibeShare/probe")
        .body(Body::empty())
        .ok()?;

    let response = tokio::time::timeout(Duration::from_millis(600), sender.send_request(request))
        .await
        .ok()?
        .ok()?;
    let is_html = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("html"))
        .unwrap_or(false);

    let mut title = None;
    if is_html {
        if let Ok(Ok(bytes)) = tokio::time::timeout(
            Duration::from_millis(600),
            http_body_util::BodyExt::collect(response.into_body()),
        )
        .await
        {
            let bytes = bytes.to_bytes();
            let head = &bytes[..bytes.len().min(16 * 1024)];
            title = extract_title(&String::from_utf8_lossy(head));
        }
    }
    driver.abort();

    Some(DevServer {
        port,
        title: title.unwrap_or_else(|| format!("端口 {port}")),
    })
}

/// 并发探测常见端口。`busy` 是 VibeShare 自己占用的端口，必须排除，
/// 否则代理到自身会造成无限循环。
pub async fn detect(busy: &[u16]) -> Vec<DevServer> {
    let targets: Vec<u16> = CANDIDATE_PORTS
        .iter()
        .copied()
        .filter(|port| !busy.contains(port))
        .collect();
    let mut found = Vec::new();
    let mut tasks = Vec::new();
    for port in targets {
        tasks.push(tokio::spawn(async move { probe(port).await }));
    }
    for task in tasks {
        if let Ok(Some(server)) = task.await {
            found.push(server);
        }
    }
    found.sort_by_key(|server| server.port);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_title() {
        assert_eq!(extract_title("<html><head><title>Vite App</title>"), Some("Vite App".into()));
        assert_eq!(extract_title("<TITLE >  Demo  </TITLE>"), Some("Demo".into()));
        assert_eq!(extract_title("<html><body>no title</body>"), None);
        assert_eq!(extract_title("<title></title>"), None);
    }

    #[test]
    fn strips_hop_by_hop_case_insensitively() {
        assert!(is_hop_by_hop("Transfer-Encoding"));
        assert!(is_hop_by_hop("connection"));
        assert!(!is_hop_by_hop("content-type"));
    }
}
