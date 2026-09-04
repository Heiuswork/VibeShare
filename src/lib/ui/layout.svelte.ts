export type LayoutMode = "split" | "full" | "window";

export const appLayout = $state({
  mode: "window" as LayoutMode,
  scale: 1,
  width: 1390,
  height: 820,
  /** 沉浸模式：只留网页本身，侧栏和预览工具条全部收掉。 */
  immersive: false,
});

function detectMode(width: number, height: number, fullscreen: boolean): LayoutMode {
  const screenWidth = window.screen?.availWidth || width;
  const screenHeight = window.screen?.availHeight || height;
  const fillsWidth = width >= screenWidth * 0.9;
  const fillsHeight = height >= screenHeight * 0.88;
  if (fullscreen || (fillsWidth && fillsHeight)) return "full";
  const tiled = fillsHeight && width <= screenWidth * 0.62;
  if (tiled || width < 1020) return "split";
  return "window";
}

function applyLayout(fullscreen = false) {
  const width = Math.max(1, window.innerWidth);
  const height = Math.max(1, window.innerHeight);
  const mode = detectMode(width, height, fullscreen);
  const scale = Math.min(1.08, Math.max(0.84, Math.min(width / 1280, height / 760)));
  appLayout.width = width;
  appLayout.height = height;
  appLayout.mode = mode;
  appLayout.scale = scale;

  const root = document.documentElement;
  root.dataset.layout = mode;
  root.style.setProperty("--ui-scale", scale.toFixed(3));
}

export function bindLayout() {
  let fullscreen = false;
  const syncFromViewport = () => applyLayout(fullscreen);

  applyLayout(fullscreen);
  window.addEventListener("resize", syncFromViewport);
  window.visualViewport?.addEventListener("resize", syncFromViewport);

  const stopFns: Array<() => void> = [];
  import("@tauri-apps/api/window")
    .then(async ({ getCurrentWindow }) => {
      const current = getCurrentWindow();
      const sync = async () => {
        fullscreen = await current.isFullscreen().catch(() => false);
        applyLayout(fullscreen);
      };
      stopFns.push(await current.onResized(() => { void sync(); }));
      await sync();
    })
    .catch(() => {});

  return () => {
    window.removeEventListener("resize", syncFromViewport);
    window.visualViewport?.removeEventListener("resize", syncFromViewport);
    stopFns.forEach((stop) => stop());
  };
}
