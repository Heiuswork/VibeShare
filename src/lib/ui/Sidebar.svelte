<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import BrandMark from "./BrandMark.svelte";
  import { t } from "./i18n";
  import { appLayout, type LayoutMode } from "./layout.svelte";
  import type { View } from "./types";

  let {
    view,
    recentCount,
    visitorCount,
    isSharing,
    networkName,
    networkIp,
    networkReachable,
    networkShortLabel,
    onView,
  }: {
    view: View;
    recentCount: number;
    visitorCount: number;
    isSharing: boolean;
    networkName: string;
    networkIp: string;
    networkReachable: boolean;
    networkShortLabel: string;
    onView: (next: View) => void;
  } = $props();

  let userPinned = $state<boolean | null>(null);
  let lastMode = $state<LayoutMode>(appLayout.mode);

  const collapsed = $derived(userPinned ?? appLayout.mode === "split");

  $effect(() => {
    const mode = appLayout.mode;
    if (mode !== lastMode) {
      userPinned = null;
      lastMode = mode;
    }
  });

  function toggleCollapsed() {
    userPinned = !collapsed;
  }
</script>

<aside class="sidebar" class:collapsed>
  <div class="brand-lockup">
    <div class="brand-icon"><BrandMark size={36} /></div>
    <div class="brand-copy">
      <strong>VibeShare</strong>
    </div>
    <button
      class="sidebar-toggle"
      onclick={toggleCollapsed}
      aria-label={collapsed ? t("expandSidebar") : t("collapseSidebar")}
      title={collapsed ? t("expandSidebar") : t("collapseSidebar")}
    >
      <Icon name="chevronLeft" size={16} />
    </button>
  </div>
  <nav class="primary-nav">
    <button class="nav-item" class:active={view === "overview"} onclick={() => onView("overview")} title={t("navPreview")}>
      <Icon name="monitor" size={17} /><span>{t("navPreview")}</span>
    </button>
    <button class="nav-item" class:active={view === "visitors"} onclick={() => onView("visitors")} title={t("navVisitors")}>
      <Icon name="users" size={17} /><span>{t("navVisitors")}</span>
      {#if isSharing}
        <span class="nav-badge live" class:zero={!visitorCount}>
          {#if visitorCount}<span class="status-dot green"></span>{/if}{visitorCount}
        </span>
      {/if}
    </button>
    <button class="nav-item" class:active={view === "recent"} onclick={() => onView("recent")} title={t("navRecent")}>
      <Icon name="clock" size={17} /><span>{t("navRecent")}</span>
      {#if recentCount}<span class="nav-badge">{recentCount}</span>{/if}
    </button>
    <button class="nav-item" class:active={view === "diagnostics"} onclick={() => onView("diagnostics")} title={t("navDiagnostics")}>
      <Icon name="pulse" size={17} /><span>{t("navDiagnostics")}</span>
    </button>
    <button class="nav-item" class:active={view === "settings"} onclick={() => onView("settings")} title={t("navSettings")}>
      <Icon name="settings" size={17} /><span>{t("navSettings")}</span>
    </button>
  </nav>
  <div class="sidebar-spacer"></div>
  <div class="network-card" title={`${networkName} · ${networkIp} · ${networkShortLabel}`}>
    <div class="network-card-top">
      <span class="network-symbol">
        <Icon name="wifi" size={16} />
        <span class="status-dot {networkReachable ? 'green' : 'amber'}"></span>
      </span>
      <span class="network-live">
        <span class="status-dot {networkReachable ? 'green' : 'amber'}"></span>
        {networkReachable ? t("networkConnected") : t("networkUnreachable")}
      </span>
    </div>
    <strong>{networkName}</strong>
    <small>{networkIp} · {networkShortLabel}</small>
  </div>
  <div class="sidebar-footer">
    <span class="privacy-mark"><Icon name="shield" size={14} /></span>
    <span>{t("privacyNote")}</span>
  </div>
</aside>

<style>
  /* 共享中时访问者数量要一眼能看到，所以徽标带一个绿点，0 台也照样显示 ——
     "现在没人访问"和"我不知道有没有人访问"是两回事。 */
  .nav-badge.live {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: var(--green-soft);
    color: var(--live-text);
  }

  .nav-badge.live.zero {
    background: var(--hover-strong);
    color: var(--muted);
  }
</style>
