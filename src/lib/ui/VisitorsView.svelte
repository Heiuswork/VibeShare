<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { formatRemaining, formatSince } from "$lib/storage";
  import type { ConnectedDevice } from "$lib/api";
  import { localizeDeviceName, t } from "./i18n";
  import { appLocale } from "./i18n.svelte";
  import DeviceActions from "./DeviceActions.svelte";

  let {
    devices,
    tick = 0,
    isSharing,
    shareUrl,
    accessCode,
    expiresAt,
    onStartShare,
    onCopyInvite,
    onBlock,
    onBlacklist,
  }: {
    devices: ConnectedDevice[];
    tick?: number;
    isSharing: boolean;
    shareUrl: string;
    accessCode: string;
    expiresAt: number | null;
    onStartShare: () => void;
    onCopyInvite: () => void;
    onBlock: (ip: string) => void;
    onBlacklist: (ip: string) => void;
  } = $props();

  type DeviceIcon = "phone" | "tablet" | "monitor" | "globe";

  function iconFor(kind: string): DeviceIcon {
    if (kind === "phone") return "phone";
    if (kind === "tablet") return "tablet";
    if (kind === "desktop") return "monitor";
    return "globe";
  }

  function kindLabel(kind: string) {
    if (kind === "phone") return t("devicePhone");
    if (kind === "tablet") return t("deviceTablet");
    if (kind === "desktop") return t("deviceDesktop");
    return t("deviceUnknown");
  }

  const rows = $derived.by(() => {
    // 读一下 tick：时间戳没变，但"12 秒前"必须随外部计时器往前走。
    void tick;
    void appLocale.current;
    return devices.map((device) => ({
      ...device,
      since: formatSince(device.last_seen),
      icon: iconFor(device.kind),
      kindText: kindLabel(device.kind),
    }));
  });
</script>

<div class="full-view visitors-view">
  {#if !isSharing}
    <div class="visitors-empty-state">
      <div class="dropzone-icon"><Icon name="users" size={22} /></div>
      <strong>{t("visitorsNotSharing")}</strong>
      <p>{t("visitorsNotSharingHint")}</p>
      <button class="primary-button compact-primary" onclick={onStartShare}>
        <Icon name="zap" size={14} /> {t("startShare")}
      </button>
    </div>
  {:else}
    <div class="visitors-header">
      <div class="visitors-count">
        <span class="status-dot green"></span>
        <strong>{rows.length}</strong>
        <span>{t("devicesViewing")}</span>
      </div>
      <div class="visitors-header-meta">
        <span><Icon name="timer" size={14} /> {formatRemaining(expiresAt)}</span>
        <button class="secondary-button" onclick={onCopyInvite}>
          <Icon name="copy" size={14} /> {t("copyInvite")}
        </button>
      </div>
    </div>

    <div class="visitors-link">
      <span><Icon name="link" size={14} /></span>
      <code>{shareUrl}</code>
      {#if accessCode}<span class="visitors-code">{t("accessCodeShort")} {accessCode}</span>{/if}
    </div>

    {#if rows.length}
      <ul class="visitors-list">
        {#each rows as device (device.ip)}
          <li>
            <span class="visitors-item-icon"><Icon name={device.icon} size={18} /></span>
            <span class="visitors-item-copy">
              <strong>{localizeDeviceName(device.name)}</strong>
              <small>{device.kindText} · {device.ip}</small>
            </span>
            <span class="visitors-item-since">{device.since}</span>
            <DeviceActions
              deviceName={localizeDeviceName(device.name)}
              onBlock={() => onBlock(device.ip)}
              onBlacklist={() => onBlacklist(device.ip)}
            />
          </li>
        {/each}
      </ul>
    {:else}
      <div class="visitors-empty-state compact">
        <div class="dropzone-icon"><Icon name="wifi" size={20} /></div>
        <strong>{t("noVisitors")}</strong>
        <p>{t("noVisitorsHint")}</p>
      </div>
    {/if}

    <p class="visitors-note">
      {t("visitorsNote")}
    </p>
  {/if}
</div>

<style>
  .visitors-view {
    display: grid;
    gap: 14px;
    align-content: start;
  }

  .visitors-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .visitors-count {
    display: flex;
    align-items: baseline;
    gap: 7px;
    font-size: 13px;
    color: var(--text-label);
  }

  .visitors-count strong {
    font-size: 24px;
    color: var(--ink);
  }

  .visitors-header-meta {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: var(--text-label);
  }

  .visitors-header-meta > span {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }

  .visitors-link {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    border: 1px solid var(--line);
    border-radius: 10px;
    background: var(--inset);
    font-size: 12px;
  }

  .visitors-link code {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .visitors-code {
    padding: 2px 7px;
    border-radius: 999px;
    background: var(--blue-soft);
    color: var(--accent-text);
    letter-spacing: 1px;
  }

  .visitors-list {
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .visitors-list li {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--panel);
  }

  .visitors-item-icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: var(--accent-fill);
    color: var(--blue);
  }

  .visitors-item-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.35;
    min-width: 0;
  }

  .visitors-item-copy small {
    font-size: 11px;
    color: var(--muted);
  }

  .visitors-item-since {
    margin-left: auto;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }

  .visitors-empty-state {
    display: grid;
    justify-items: center;
    gap: 8px;
    padding: 44px 20px;
    border: 1px dashed var(--line-strong);
    border-radius: 14px;
    text-align: center;
  }

  .visitors-empty-state.compact {
    padding: 28px 20px;
  }

  .visitors-empty-state p {
    margin: 0;
    font-size: 12px;
    color: var(--muted);
  }

  .visitors-note {
    margin: 0;
    font-size: 11px;
    color: var(--soft);
  }
</style>
