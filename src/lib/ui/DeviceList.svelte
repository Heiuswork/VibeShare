<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { formatSince } from "$lib/storage";
  import type { ConnectedDevice } from "$lib/api";
  import DeviceActions from "./DeviceActions.svelte";
  import { localizeDeviceName, t } from "./i18n";
  import { appLocale } from "./i18n.svelte";

  let {
    devices,
    tick = 0,
    compact = false,
    onOpenAll,
    onBlock,
    onBlacklist,
  }: {
    devices: ConnectedDevice[];
    tick?: number;
    compact?: boolean;
    onOpenAll?: () => void;
    onBlock?: (ip: string) => void;
    onBlacklist?: (ip: string) => void;
  } = $props();

  // 与 Icon.svelte 的 IconName 保持字面量子集，不跨组件导入类型。
  type DeviceIcon = "phone" | "tablet" | "monitor" | "globe";

  function iconFor(kind: string): DeviceIcon {
    if (kind === "phone") return "phone";
    if (kind === "tablet") return "tablet";
    if (kind === "desktop") return "monitor";
    return "globe";
  }

  const rows = $derived.by(() => {
    // 读一下 tick：时间戳没变，但"12 秒前"这句话必须随外部计时器往前走。
    void tick;
    void appLocale.current;
    return devices.map((device) => ({
      ...device,
      since: formatSince(device.last_seen),
      icon: iconFor(device.kind),
    }));
  });
</script>

<div class="online-devices-panel" class:compact>
  {#if !compact}
    <div class="online-devices-head">
      <strong>{t("onlineDevices")}</strong>
      {#if onOpenAll}
        <button type="button" class="text-button" onclick={onOpenAll}>{t("visitorsPage")} <Icon name="arrow" size={12} /></button>
      {:else}
        <small>{t("nameIpActivity")}</small>
      {/if}
    </div>
  {/if}
  {#if rows.length}
    <ul class="online-devices-list">
      {#each rows as device (device.ip)}
        <li>
          <span class="device-icon"><Icon name={device.icon} size={14} /></span>
          <span class="device-copy">
            <strong>{localizeDeviceName(device.name)}</strong>
            <small>{device.ip}</small>
          </span>
          <span class="device-since">{device.since}</span>
          {#if onBlock && onBlacklist}
            <DeviceActions
              deviceName={localizeDeviceName(device.name)}
              onBlock={() => onBlock(device.ip)}
              onBlacklist={() => onBlacklist(device.ip)}
            />
          {/if}
        </li>
      {/each}
    </ul>
  {:else}
    <div class="online-devices-empty">{t("noOtherDevices")}</div>
  {/if}
  {#if compact && onOpenAll}
    <button type="button" class="text-button device-list-more" onclick={onOpenAll}>
      {t("viewAllOnVisitors")} <Icon name="arrow" size={12} />
    </button>
  {/if}
</div>

<style>
  .device-since {
    margin-left: auto;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  /* 嵌在别的面板里时不带自己的外框和标题，限高交给外面那层容器。 */
  .online-devices-panel.compact {
    margin: 0;
    padding: 0;
    border: 0;
    background: transparent;
  }

  .device-list-more {
    margin-top: 6px;
  }
</style>
