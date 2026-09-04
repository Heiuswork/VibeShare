<script lang="ts">
  /**
   * 在线设备是一张独立的浮层面板，不再摊在共享面板里 ——
   * 设备一多，原来的做法会把二维码和访问码整片往下顶。
   * 它固定在预览区左上角，和右上角的共享面板互不遮挡。
   */
  import Icon from "$lib/Icon.svelte";
  import type { ConnectedDevice } from "$lib/api";
  import DeviceList from "./DeviceList.svelte";
  import { t } from "./i18n";

  let {
    devices,
    tick = 0,
    onClose,
    onOpenVisitors,
    onBlock,
    onBlacklist,
  }: {
    devices: ConnectedDevice[];
    tick?: number;
    onClose: () => void;
    onOpenVisitors: () => void;
    onBlock: (ip: string) => void;
    onBlacklist: (ip: string) => void;
  } = $props();

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="device-panel" role="dialog" aria-label={t("onlineDevices")}>
  <div class="device-panel-head">
    <div class="device-panel-title">
      <strong>{t("onlineDevices")}</strong>
      <span class="device-panel-count">{devices.length}</span>
    </div>
    <button class="icon-button" onclick={onClose} aria-label={t("closeDevices")} title={t("close")}>
      <Icon name="x" size={14} />
    </button>
  </div>
  <div class="device-panel-body">
    <DeviceList {devices} {tick} compact {onBlock} {onBlacklist} />
  </div>
  <button type="button" class="text-button device-panel-more" onclick={onOpenVisitors}>
    {t("visitorsPage")} <Icon name="arrow" size={12} />
  </button>
</div>

<style>
  /* 紧贴共享面板的左边缘：共享面板 right:14px、宽 340px，
     所以这里的 right = 14 + 340 + 10 = 364px，两张面板头部齐平。
     窗口窄到放不下时退到距左边 14px，不会被切掉。 */
  .device-panel {
    position: absolute;
    top: 14px;
    right: min(364px, calc(100% - 262px));
    z-index: 19;
    width: 248px;
    padding: 11px 12px 9px;
    border: 1px solid var(--line-strong);
    border-radius: 12px;
    background: var(--overlay);
    box-shadow: var(--menu-shadow);
    backdrop-filter: blur(14px);
    animation: panel-in 0.16s ease-out;
    pointer-events: auto;
    -webkit-app-region: no-drag;
  }

  .device-panel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--line);
  }

  .device-panel-title {
    display: inline-flex;
    align-items: center;
    gap: 7px;
  }

  .device-panel-title strong {
    color: var(--text-strong);
    font-size: 12px;
  }

  /* 数量做成一颗小计数，跟侧边栏"访问者"上的徽标是同一套语汇。 */
  .device-panel-count {
    min-width: 17px;
    padding: 1px 5px;
    color: var(--green);
    background: var(--green-soft);
    border-radius: 999px;
    font-size: 10px;
    font-weight: 650;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .device-panel-body {
    max-height: 236px;
    overflow: auto;
  }

  .device-panel-more {
    margin-top: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--line);
    width: 100%;
    justify-content: flex-end;
  }
</style>
