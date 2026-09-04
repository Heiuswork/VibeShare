<script lang="ts">
  /* 应用图标的界面版：方案 35「交集」+ 配色 03。
     Icon.svelte 全是 currentColor 单色描边，装不下冰蓝 + 薄荷这一对，所以单独一个组件。

     两套画法，和 design/render-icon.py 里的 BIG / TINY 变体逐个数字对齐（64 设计空间）：
     44px 以上用细线版，以下换粗线版并把两张页面都放大。小尺寸不是把大的缩小 ——
     3.6 的线宽到 28px 上只剩 1.5 个物理像素，两条线会并成一条。
     区别只在于这里不留 macOS 那圈留白：界面里的品牌位本身就是砖块，不进 Dock。

     knock：前面那张的加宽描边先用底色把后面那张咬断，一前一后的占位关系才读得出来。 */
  let { size = 36, radius }: { size?: number; radius?: number } = $props();

  const FINE = { back: [11, 14, 27, 27, 7.5], front: [26, 23, 27, 27, 7.5], sw: 3.6, knock: 3.2 };
  const BOLD = { back: [9, 11, 29, 29, 8.5], front: [26, 24, 29, 29, 8.5], sw: 5.6, knock: 3.0 };

  const g = $derived(size > 44 ? FINE : BOLD);
  /* 砖块越小，圆角占比越大一点，不然看起来偏方 —— 系统图标也是这么处理的。 */
  const rx = $derived(((radius ?? (size > 44 ? 22.37 : size > 24 ? 25 : 28)) / 100) * 64);
</script>

<svg class="brand-mark" width={size} height={size} viewBox="0 0 64 64" fill="none" aria-hidden="true">
  <defs>
    <linearGradient id="bmTile" x1="0.18" y1="0" x2="0.82" y2="1">
      <stop offset="0" stop-color="#1f2b3d" />
      <stop offset="1" stop-color="#0d1420" />
    </linearGradient>
    <linearGradient id="bmSheen" x1="0" y1="0" x2="0.18" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="0.18" />
      <stop offset="0.45" stop-color="#fff" stop-opacity="0.03" />
      <stop offset="1" stop-color="#fff" stop-opacity="0" />
    </linearGradient>
  </defs>
  <rect x="0" y="0" width="64" height="64" rx={rx} fill="url(#bmTile)" />
  <rect
    x={g.back[0]}
    y={g.back[1]}
    width={g.back[2]}
    height={g.back[3]}
    rx={g.back[4]}
    stroke="#7fb2ff"
    stroke-width={g.sw}
  />
  <rect
    x={g.front[0]}
    y={g.front[1]}
    width={g.front[2]}
    height={g.front[3]}
    rx={g.front[4]}
    stroke="#16202f"
    stroke-width={g.sw + g.knock}
  />
  <rect
    x={g.front[0]}
    y={g.front[1]}
    width={g.front[2]}
    height={g.front[3]}
    rx={g.front[4]}
    stroke="#7ee7c7"
    stroke-width={g.sw}
  />
  <rect x="0" y="0" width="64" height="64" rx={rx} fill="url(#bmSheen)" />
  <rect x="0.6" y="0.6" width="62.8" height="62.8" rx={rx - 0.6} stroke="#fff" stroke-opacity="0.1" />
</svg>

<style>
  .brand-mark {
    display: block;
    flex: 0 0 auto;
  }
</style>
