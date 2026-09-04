//! 资源引用诊断。
//!
//! 原先"静态资源"检查项恒定显示"目录直读"，等于没有检查。这里真正扫描入口
//! HTML 及其引用的 CSS，找出两类在手机上必然失败、但浏览器报错完全不指向
//! 原因的问题：
//!
//! 1. 指向共享根下不存在文件的引用（常见于把绝对路径当成站点根）。
//! 2. 硬编码的 `http://localhost:PORT` 引用 —— AI 生成页面的高频产物，
//!    在分享者机器上正常，访问者那边一定连不上。

use percent_encoding::percent_decode_str;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// 样本上限。诊断只需要让用户看到问题长什么样，不需要穷举。
const MAX_SAMPLES: usize = 12;
/// 顺带扫描的 CSS 文件数量上限。
const MAX_STYLESHEETS: usize = 8;

#[derive(Debug, Clone, Default, Serialize)]
pub struct AssetReport {
    /// 实际检查过的本地引用数量。
    pub scanned: usize,
    /// 指向不存在文件的引用。
    pub missing: Vec<String>,
    /// 硬编码到本机地址的引用。
    pub hardcoded_local: Vec<String>,
    /// 样本是否被截断。
    pub truncated: bool,
}

/// HTML 中可能携带资源路径的属性。
const URL_ATTRS: [&str; 6] = ["src", "href", "srcset", "poster", "data-src", "content"];

fn is_name_boundary(ch: u8) -> bool {
    !(ch.is_ascii_alphanumeric() || ch == b'-' || ch == b'_' || ch == b':')
}

/// 抽出指定属性的值。手写扫描而不是引入 HTML 解析器：诊断只需要够用的近似，
/// 漏掉少数畸形写法比多一个重量级依赖划算。
fn attribute_values(text: &str, attrs: &[&str]) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let mut found = Vec::new();
    for attr in attrs {
        let mut from = 0usize;
        while let Some(offset) = lower[from..].find(attr) {
            let start = from + offset;
            from = start + attr.len();
            // 前一个字符必须是边界，否则 `data-src` 会被当成 `src`。
            if start > 0 && !is_name_boundary(bytes[start - 1]) {
                continue;
            }
            let mut cursor = start + attr.len();
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor >= bytes.len() || bytes[cursor] != b'=' {
                continue;
            }
            cursor += 1;
            while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if cursor >= bytes.len() {
                break;
            }
            let quote = bytes[cursor];
            let (value_start, value_end) = if quote == b'"' || quote == b'\'' {
                let value_start = cursor + 1;
                match lower[value_start..].find(quote as char) {
                    Some(length) => (value_start, value_start + length),
                    None => break,
                }
            } else {
                let value_start = cursor;
                let end = lower[value_start..]
                    .find(|ch: char| ch.is_whitespace() || ch == '>')
                    .map(|length| value_start + length)
                    .unwrap_or(bytes.len());
                (value_start, end)
            };
            if value_end > value_start {
                found.push(text[value_start..value_end].to_string());
            }
            from = value_end;
        }
    }
    found
}

/// 抽出 CSS 里 `url(...)` 的目标。
fn css_urls(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let lower = text.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(offset) = lower[from..].find("url(") {
        let start = from + offset + 4;
        let Some(length) = text[start..].find(')') else {
            break;
        };
        let raw = text[start..start + length].trim().trim_matches('"').trim_matches('\'');
        if !raw.is_empty() {
            found.push(raw.to_string());
        }
        from = start + length;
    }
    found
}

/// `srcset` 是 `url 1x, url 2x` 形式，取每段的第一个 token。
fn expand_srcset(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter_map(|part| part.trim().split_whitespace().next())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

enum Reference {
    /// 与本地文件无关，无需检查。
    External,
    /// 硬编码到本机地址。
    HardcodedLocal,
    /// 需要在共享根下解析的相对或绝对路径。
    Local(String),
}

fn points_at_local_host(url: &str) -> bool {
    let rest = url.split("//").nth(1).unwrap_or("");
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    let bare = host.split(':').next().unwrap_or(host);
    matches!(bare, "localhost" | "127.0.0.1" | "0.0.0.0") || host.starts_with("[::1]")
}

fn classify(raw: &str) -> Reference {
    let value = raw.trim();
    if value.is_empty() || value.starts_with('#') {
        return Reference::External;
    }
    let lower = value.to_ascii_lowercase();
    for prefix in ["data:", "mailto:", "tel:", "javascript:", "blob:", "about:", "sms:", "ws:", "wss:"] {
        if lower.starts_with(prefix) {
            return Reference::External;
        }
    }
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return if points_at_local_host(&lower) {
            Reference::HardcodedLocal
        } else {
            Reference::External
        };
    }
    if lower.starts_with("//") {
        return Reference::External;
    }
    // 去掉 query 与 hash，只留路径部分。
    let path = value.split(['?', '#']).next().unwrap_or(value);
    if path.is_empty() {
        return Reference::External;
    }
    Reference::Local(path.to_string())
}

/// 把引用解析成磁盘路径，并确认没有逃出共享根。
fn resolve(root: &Path, base: &Path, reference: &str) -> Option<PathBuf> {
    let decoded = percent_decode_str(reference).decode_utf8_lossy().to_string();
    let candidate = if let Some(stripped) = decoded.strip_prefix('/') {
        root.join(stripped)
    } else {
        base.join(&decoded)
    };
    // 手工归一化 `.` 与 `..`，避免对不存在的路径调用 canonicalize。
    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            std::path::Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            std::path::Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    if !normalized.starts_with(root) {
        return None;
    }
    Some(normalized)
}

struct Collector {
    scanned: usize,
    missing: Vec<String>,
    hardcoded: Vec<String>,
    truncated: bool,
}

impl Collector {
    fn note_missing(&mut self, reference: &str) {
        if self.missing.iter().any(|item| item == reference) {
            return;
        }
        if self.missing.len() >= MAX_SAMPLES {
            self.truncated = true;
            return;
        }
        self.missing.push(reference.to_string());
    }

    fn note_hardcoded(&mut self, reference: &str) {
        if self.hardcoded.iter().any(|item| item == reference) {
            return;
        }
        if self.hardcoded.len() >= MAX_SAMPLES {
            self.truncated = true;
            return;
        }
        self.hardcoded.push(reference.to_string());
    }
}

/// 扫描入口 HTML 及其引用的样式表。
pub fn scan(root: &Path, entry: &str) -> AssetReport {
    let entry_path = root.join(entry);
    let base = entry_path.parent().map(Path::to_path_buf).unwrap_or_else(|| root.to_path_buf());
    let Ok(html) = std::fs::read_to_string(&entry_path) else {
        return AssetReport::default();
    };

    let mut collector = Collector {
        scanned: 0,
        missing: Vec::new(),
        hardcoded: Vec::new(),
        truncated: false,
    };
    let mut stylesheets: Vec<PathBuf> = Vec::new();

    let mut references: Vec<String> = Vec::new();
    for value in attribute_values(&html, &URL_ATTRS) {
        if value.contains(',') && value.split_whitespace().count() > 1 {
            references.extend(expand_srcset(&value));
        } else {
            references.push(value);
        }
    }
    references.extend(css_urls(&html));

    for reference in &references {
        match classify(reference) {
            Reference::External => {}
            Reference::HardcodedLocal => collector.note_hardcoded(reference),
            Reference::Local(path) => {
                collector.scanned += 1;
                match resolve(root, &base, &path) {
                    Some(resolved) => {
                        if resolved.extension().and_then(|value| value.to_str()) == Some("css")
                            && resolved.is_file()
                            && stylesheets.len() < MAX_STYLESHEETS
                        {
                            stylesheets.push(resolved.clone());
                        }
                        // 目录型引用（如 `/about/`）交给 SPA 回退处理，不算缺失。
                        if !resolved.exists() && !path.ends_with('/') {
                            collector.note_missing(reference);
                        }
                    }
                    None => collector.note_missing(reference),
                }
            }
        }
    }

    for sheet in stylesheets {
        let Ok(css) = std::fs::read_to_string(&sheet) else {
            continue;
        };
        let sheet_base = sheet.parent().map(Path::to_path_buf).unwrap_or_else(|| root.to_path_buf());
        for reference in css_urls(&css) {
            match classify(&reference) {
                Reference::External => {}
                Reference::HardcodedLocal => collector.note_hardcoded(&reference),
                Reference::Local(path) => {
                    collector.scanned += 1;
                    match resolve(root, &sheet_base, &path) {
                        Some(resolved) => {
                            if !resolved.exists() {
                                collector.note_missing(&reference);
                            }
                        }
                        None => collector.note_missing(&reference),
                    }
                }
            }
        }
    }

    AssetReport {
        scanned: collector.scanned,
        missing: collector.missing,
        hardcoded_local: collector.hardcoded,
        truncated: collector.truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_attributes_without_matching_suffixes() {
        let html = r#"<img src="a.png" data-src="b.png"><a href='c.html'>"#;
        let values = attribute_values(html, &["src"]);
        assert_eq!(values, vec!["a.png".to_string()]);
        let values = attribute_values(html, &["href"]);
        assert_eq!(values, vec!["c.html".to_string()]);
    }

    #[test]
    fn detects_hardcoded_local_hosts() {
        assert!(matches!(classify("http://localhost:3000/api"), Reference::HardcodedLocal));
        assert!(matches!(classify("http://127.0.0.1:8080/x.js"), Reference::HardcodedLocal));
        assert!(matches!(classify("https://cdn.example.com/x.js"), Reference::External));
        assert!(matches!(classify("//cdn.example.com/x.js"), Reference::External));
        assert!(matches!(classify("data:image/png;base64,AAA"), Reference::External));
    }

    #[test]
    fn strips_query_and_hash() {
        match classify("app.js?v=2#frag") {
            Reference::Local(path) => assert_eq!(path, "app.js"),
            _ => panic!("expected local reference"),
        }
    }

    #[test]
    fn rejects_escapes_above_root() {
        let root = Path::new("/site");
        assert!(resolve(root, Path::new("/site"), "../secret.txt").is_none());
        assert_eq!(
            resolve(root, Path::new("/site/pages"), "../app.css"),
            Some(PathBuf::from("/site/app.css"))
        );
    }

    #[test]
    fn reads_css_urls() {
        let css = "a{background:url('img/x.png')} b{background:url(y.png)}";
        assert_eq!(css_urls(css), vec!["img/x.png".to_string(), "y.png".to_string()]);
    }

    #[test]
    fn expands_srcset() {
        assert_eq!(
            expand_srcset("a.png 1x, b.png 2x"),
            vec!["a.png".to_string(), "b.png".to_string()]
        );
    }
}
