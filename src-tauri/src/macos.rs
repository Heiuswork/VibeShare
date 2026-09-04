use std::ffi::c_void;

use objc2::MainThreadMarker;
use objc2_app_kit::NSWindow;
use objc2_foundation::NSSize;
use tauri::WebviewWindow;

/// 必须远小于半屏，否则系统会拒绝进入全屏分屏（磁贴放不下）。
const MIN_TILE_WIDTH_PT: f64 = 240.0;
const MIN_TILE_HEIGHT_PT: f64 = 200.0;

pub fn install(window: &WebviewWindow) {
    apply_fullscreen_tile_limits(window);
}

/// 只改分屏用的最小内容尺寸。不要改 collectionBehavior，系统默认就允许分屏；
/// 也不要写 max / 普通 minSize，更不要按屏幕比例计算（2x 屏会变成半屏下限，进而无法分屏）。
pub fn apply_fullscreen_tile_limits(window: &WebviewWindow) {
    let pending = window.clone();
    if let Err(error) = window.with_webview(move |webview| unsafe {
        let ns_window: &NSWindow = &*webview.ns_window().cast();
        patch_ns_window(ns_window, "with_webview");
    }) {
        eprintln!("[vibeshare] with_webview failed: {error}; falling back to ns_window()");
        apply_from_handle(&pending);
    }
}

fn apply_from_handle(window: &WebviewWindow) {
    let Ok(ptr) = window.ns_window() else {
        eprintln!("[vibeshare] ns_window() unavailable");
        return;
    };
    if ptr.is_null() {
        eprintln!("[vibeshare] ns_window() was null");
        return;
    }
    unsafe {
        let ns_window: &NSWindow = &*ptr.cast::<c_void>().cast();
        patch_ns_window(ns_window, "ns_window");
    }
}

unsafe fn patch_ns_window(ns_window: &NSWindow, source: &str) {
    if MainThreadMarker::new().is_none() {
        eprintln!("[vibeshare] skip {source}: not on main thread");
        return;
    }

    let min_size = NSSize::new(MIN_TILE_WIDTH_PT, MIN_TILE_HEIGHT_PT);
    ns_window.setContentMinSize(min_size);
    ns_window.setMinFullScreenContentSize(min_size);

    let applied = ns_window.minFullScreenContentSize();
    eprintln!(
        "[vibeshare] split limits via {source}: minFS={:.0}x{:.0} contentMin={:.0}x{:.0}",
        applied.width,
        applied.height,
        ns_window.contentMinSize().width,
        ns_window.contentMinSize().height,
    );
}
