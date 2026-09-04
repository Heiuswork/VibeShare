//! 站点文件监听。
//!
//! 取代原先每 500ms 遍历目录计算 stamp 的做法：改用系统文件事件
//! （macOS 上是 FSEvents），只在真正有改动时递增版本号。
//! 版本号由 `/__vibeshare/revision` 暴露，注入页面的脚本轮询它来决定是否刷新。

use notify::{EventKind, RecursiveMode, Watcher};
use std::path::{Component, Path};

/// 遍历与监听时都跳过的目录名。
///
/// 注意这里不含 `dist` / `build` / `out`：构建产物目录恰恰是最常被共享的站点根，
/// 把它们排除会导致扫不到入口、改动也监听不到。
const IGNORED_DIRS: [&str; 12] = [
    "node_modules",
    ".git",
    ".svelte-kit",
    "target",
    "vendor",
    "Pods",
    ".next",
    ".nuxt",
    "coverage",
    "__pycache__",
    "Library",
    "Applications",
];

pub fn is_ignored_dir(name: &str) -> bool {
    IGNORED_DIRS.contains(&name)
}

pub fn is_hidden(name: &str) -> bool {
    name.starts_with('.')
}

/// 编辑器保存时常见的中间文件，忽略以免无意义地触发刷新。
fn is_temp_file(name: &str) -> bool {
    name.ends_with('~')
        || name.ends_with(".swp")
        || name.ends_with(".swx")
        || name.ends_with(".tmp")
        || name.starts_with(".#")
        || name.starts_with("~$")
}

/// 判断某个事件路径是否应该被忽略。
fn is_ignored_path(path: &Path) -> bool {
    let mut components = path.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(raw) = component else {
            continue;
        };
        let name = raw.to_string_lossy();
        let last = components.peek().is_none();
        if last {
            if is_temp_file(&name) || is_hidden(&name) {
                return true;
            }
        } else if is_ignored_dir(&name) || is_hidden(&name) {
            return true;
        }
    }
    false
}

/// 持有 watcher 句柄。drop 即停止监听，所以必须存进状态里。
pub struct SiteWatcher {
    _watcher: notify::RecommendedWatcher,
}

/// 递归监听 `root`，有相关改动时调用 `on_change`。
///
/// `on_change` 可能被高频调用（一次保存往往产生多个事件），调用方只需
/// 递增计数器即可 —— 客户端轮询天然会把连续变化合并成一次刷新。
pub fn watch<F>(root: &Path, on_change: F) -> Option<SiteWatcher>
where
    F: Fn() + Send + 'static,
{
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else {
            return;
        };
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            return;
        }
        if event.paths.iter().all(|path| is_ignored_path(path)) {
            return;
        }
        on_change();
    })
    .ok()?;
    watcher.watch(root, RecursiveMode::Recursive).ok()?;
    Some(SiteWatcher { _watcher: watcher })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn ignores_dependency_and_vcs_dirs() {
        assert!(is_ignored_path(&PathBuf::from("/site/node_modules/react/index.js")));
        assert!(is_ignored_path(&PathBuf::from("/site/.git/HEAD")));
    }

    #[test]
    fn does_not_ignore_build_output() {
        assert!(!is_ignored_path(&PathBuf::from("/site/dist/index.html")));
        assert!(!is_ignored_path(&PathBuf::from("/site/build/app.css")));
    }

    #[test]
    fn ignores_editor_temp_files() {
        assert!(is_ignored_path(&PathBuf::from("/site/index.html~")));
        assert!(is_ignored_path(&PathBuf::from("/site/.index.html.swp")));
        assert!(!is_ignored_path(&PathBuf::from("/site/index.html")));
    }
}
