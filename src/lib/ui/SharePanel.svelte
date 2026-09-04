<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { expiryDisplayLabel, formatRemaining } from "$lib/storage";
  import type { ShareStatus, SiteInfo } from "$lib/api";
  import CheckRow from "./CheckRow.svelte";
  import ExpiryPicker from "./ExpiryPicker.svelte";
  import type { CheckItem, ChecksSummary } from "./types";
  import { t } from "./i18n";

  let {
    site,
    status,
    isSharing,
    starting,
    checking,
    checksExpanded = $bindable(),
    checks,
    checkItems,
    qr,
    previewQr,
    shareUrl,
    shareExpiry,
    shareCustom,
    shareCustomHours = $bindable(),
    shareCustomMinutes = $bindable(),
    accessProtection,
    onClose,
    onToggleChecks,
    onRecheck,
    onChooseEntry,
    onStart,
    onStop,
    onResetCode,
    onCopyLink,
    onCopyInvite,
    onPreset,
    onApplyCustom,
    onToggleDevices,
    devicesOpen = false,
  }: {
    site: SiteInfo | null;
    status: ShareStatus | null;
    isSharing: boolean;
    starting: boolean;
    checking: boolean;
    checksExpanded: boolean;
    checks: ChecksSummary;
    checkItems: CheckItem[];
    qr: string;
    previewQr: string;
    shareUrl: string;
    shareExpiry: string;
    shareCustom: boolean;
    shareCustomHours: number;
    shareCustomMinutes: number;
    accessProtection: boolean;
    onClose: () => void;
    onToggleChecks: () => void;
    onRecheck: () => void;
    onChooseEntry: () => void;
    onStart: () => void;
    onStop: () => void;
    onResetCode: () => void;
    onCopyLink: () => void;
    onCopyInvite: () => void;
    onPreset: (value: string) => void;
    onApplyCustom: () => void;
    onToggleDevices: () => void;
    devicesOpen?: boolean;
  } = $props();

  const devices = $derived(status?.devices ?? []);
</script>

<div class="share-panel" data-share-panel>
  <div class="share-panel-head">
    <div><strong>{t("share")}</strong></div>
    <button class="icon-button" onclick={onClose} aria-label={t("closeSharePanel")} title={t("close")}>
      <Icon name="x" size={16} />
    </button>
  </div>

  <div class="floating-checks" class:expanded={checksExpanded} class:collapsed={!checksExpanded}>
    <button class="checks-summary" onclick={onToggleChecks} aria-expanded={checksExpanded}>
      <span class="checks-summary-left">
        <span class="checks-tone {checks.tone}">
          {#if checks.tone === "checking"}
            <span class="spinner tiny"></span>
          {:else}
            <Icon name={checks.tone === "ok" ? "check" : "alert"} size={14} />
          {/if}
        </span>
        <span><strong>{t("checks")}</strong><small>{checks.label}</small></span>
      </span>
      <span class="checks-summary-right">
        <span class="checks-count {checks.tone}">{checking ? "…" : `${checks.okCount}/${checks.total}`}</span>
        <span class="checks-chevron" class:up={checksExpanded}><Icon name="chevron" size={14} /></span>
      </span>
    </button>
    {#if checksExpanded}
      <div class="floating-check-list">
        {#if checking}
          <div class="checking-state compact"><span class="spinner"></span><span>{t("checking")}</span></div>
        {:else}
          {#each checkItems as item}
            <CheckRow
              label={item.label}
              detail={item.detail}
              status={item.status}
              action={item.action}
              onAction={onChooseEntry}
            />
          {/each}
        {/if}
        <div class="checks-actions">
          <button class="text-button refresh-button" onclick={onRecheck}><Icon name="refresh" size={14} /> {t("recheck")}</button>
        </div>
      </div>
    {/if}
  </div>

  {#if isSharing}
    <div class="active-share-card floating-share-body">
      <div class="active-share-top">
        <div><h3>{t("sharingLive")}</h3></div>
        <span class="live-badge"><span class="status-dot green"></span> {t("online")}</span>
      </div>
      <div class="active-meta between-title-qr">
        <!-- "N 台在线"只干一件事：开关左边那张独立的在线设备面板。
             列表不进这张卡片，否则设备一多就把二维码顶下去。 -->
        <button
          type="button"
          class="devices-toggle"
          class:open={devicesOpen}
          onclick={onToggleDevices}
          aria-expanded={devicesOpen}
          title={devicesOpen ? t("collapseDevices") : t("viewDevices")}
        >
          <Icon name="users" size={14} /> {t("devicesOnline", { n: devices.length })}
          <span class="devices-chevron" class:up={devicesOpen}><Icon name="chevron" size={12} /></span>
        </button>
        <span><Icon name="timer" size={14} /> {formatRemaining(status?.expires_at)}</span>
      </div>
      <div class="qr-stage">
        <div class="qr-code" role="img" aria-label={t("shareQr")}>
          {#if qr}<img src={qr} alt="" />{/if}
        </div>
        <span class="qr-caption">{t("scanToOpen")}</span>
      </div>
      <div class="code-row">
        <div>
          <span class="metric-label">{t("accessCodeShort")}</span>
          <strong class="access-code">{status?.access_code || t("accessCodeOff")}</strong>
        </div>
        <button class="secondary-button code-copy" onclick={onResetCode} disabled={!status?.access_code}>
          <Icon name="refresh" size={14} /> {t("reset")}
        </button>
      </div>
      <div class="url-field compact-url">
        <span><Icon name="link" size={14} /></span>
        <code>{shareUrl}</code>
        <button class="icon-button" onclick={onCopyLink} aria-label={t("copyShareUrl")} title={t("copyShareUrl")}>
          <Icon name="copy" size={14} />
        </button>
      </div>
      <button class="invite-button" onclick={onCopyInvite}>
        <Icon name="qr" size={16} /> {t("copyInviteFull")} <span><Icon name="arrow" size={15} /></span>
      </button>
      <button class="stop-button" onclick={onStop}><Icon name="square" size={14} /> {t("stopShare")}</button>
    </div>
  {:else}
    <div class="start-card floating-share-body">
      <div class="start-card-head">
        <strong>{t("startShare")}</strong>
        <span class="ready-icon"><Icon name="zap" size={16} /></span>
      </div>
      <div class="preview-qr-wrap">
        <div class="qr-code muted" role="img" aria-label={t("shareQr")}>
          {#if previewQr}<img src={previewQr} alt="" />{/if}
        </div>
        <div class="preview-lock"><Icon name="lock" size={14} /> {t("showAfterShare")}</div>
      </div>
      {#if !site}
        <div class="start-copy"><p>{t("chooseSiteFirst")}</p></div>
      {/if}
      <ExpiryPicker
        compact
        presets
        label={t("duration")}
        value={shareExpiry}
        custom={shareCustom}
        bind:customHours={shareCustomHours}
        bind:customMinutes={shareCustomMinutes}
        onPreset={onPreset}
        onApply={onApplyCustom}
      />
      <button class="primary-button start-button" onclick={onStart} disabled={!site || starting}>
        {#if starting}<span class="spinner"></span>{:else}<Icon name="zap" size={17} />{/if}
        {starting ? t("preparingShare") : t("startShare")}
        <span class="button-arrow"><Icon name="arrow" size={16} /></span>
      </button>
      <div class="secure-caption">
        <Icon name="shield" size={13} /> {accessProtection ? t("accessCodeOn") : t("accessCodeOffStatus")} · {expiryDisplayLabel(shareExpiry)}
      </div>
    </div>
  {/if}
</div>

<style>
  /* 和旁边"剩余 30 分钟"同一行、同字号、同颜色，只是可点。
     不给它配色块，hover 才浮出一条细边 —— 面板里已经有足够多的框了。 */
  .devices-toggle {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 7px;
    margin: -3px -7px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    transition: background 0.16s, border-color 0.16s;
  }

  .devices-toggle :global(.icon) {
    color: var(--green);
  }

  .devices-toggle:hover,
  .devices-toggle.open {
    border-color: var(--live-line);
    background: var(--panel);
  }

  .devices-chevron {
    display: inline-flex;
    transition: transform 0.16s ease;
  }

  .devices-chevron.up {
    transform: rotate(180deg);
  }
</style>
