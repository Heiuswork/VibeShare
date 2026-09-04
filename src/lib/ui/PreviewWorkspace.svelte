<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { RecentItem } from "$lib/storage";
  import type { DevServer, ShareStatus, SiteInfo } from "$lib/api";
  import EntryPicker from "./EntryPicker.svelte";
  import ModePicker from "./ModePicker.svelte";
  import SharePanel from "./SharePanel.svelte";
  import DevicePanel from "./DevicePanel.svelte";
  import type { CheckItem, ChecksSummary, ShareButtonState } from "./types";
  import PreviewFrame from "./PreviewFrame.svelte";
  import { localizeSiteName, localizeUpdated, t } from "./i18n";
  import { appLayout } from "./layout.svelte";

  let {
    site,
    status,
    tick = 0,
    devServerList,
    modeBusy,
    busy,
    starting,
    checking,
    dragging,
    previewUrl,
    previewQr,
    qr,
    shareUrl,
    showSharePanel,
    showEntryPicker,
    checksExpanded = $bindable(),
    shareButton,
    checks,
    checkItems,
    shareExpiry,
    shareCustom,
    shareCustomHours = $bindable(),
    shareCustomMinutes = $bindable(),
    emptyRecent,
    recentCount,
    isSharing,
    accessProtection,
    onChooseFolder,
    onChooseHtmlFile,
    onRefreshPreview,
    onToggleSharePanel,
    onReopenRecent,
    onGoRecent,
    onSelectEntry,
    onCloseEntryPicker,
    onCloseSharePanel,
    onToggleChecks,
    onRecheck,
    onChooseEntry,
    onStartShare,
    onStopShare,
    onResetCode,
    onCopyLink,
    onCopyInvite,
    onSharePreset,
    onApplyShareCustom,
    onScanDevServers,
    onUseDevServer,
    onUseStatic,
    onOpenVisitors,
    onBlock,
    onBlacklist,
  }: {
    site: SiteInfo | null;
    status: ShareStatus | null;
    tick?: number;
    devServerList: DevServer[];
    modeBusy: boolean;
    busy: boolean;
    starting: boolean;
    checking: boolean;
    dragging: boolean;
    previewUrl: string;
    previewQr: string;
    qr: string;
    shareUrl: string;
    showSharePanel: boolean;
    showEntryPicker: boolean;
    checksExpanded: boolean;
    shareButton: ShareButtonState;
    checks: ChecksSummary;
    checkItems: CheckItem[];
    shareExpiry: string;
    shareCustom: boolean;
    shareCustomHours: number;
    shareCustomMinutes: number;
    emptyRecent: RecentItem[];
    recentCount: number;
    isSharing: boolean;
    accessProtection: boolean;
    onChooseFolder: () => void;
    onChooseHtmlFile: () => void;
    onRefreshPreview: () => void;
    onToggleSharePanel: () => void;
    onReopenRecent: (item: RecentItem) => void;
    onGoRecent: () => void;
    onSelectEntry: (entry: string) => void;
    onCloseEntryPicker: () => void;
    onCloseSharePanel: () => void;
    onToggleChecks: () => void;
    onRecheck: () => void;
    onChooseEntry: () => void;
    onStartShare: () => void;
    onStopShare: () => void;
    onResetCode: () => void;
    onCopyLink: () => void;
    onCopyInvite: () => void;
    onSharePreset: (value: string) => void;
    onApplyShareCustom: () => void;
    onScanDevServers: () => void;
    onUseDevServer: (port: number) => void;
    onUseStatic: () => void;
    onOpenVisitors: () => void;
    onBlock: (ip: string) => void;
    onBlacklist: (ip: string) => void;
  } = $props();

  const mode = $derived(status?.mode ?? "static");
  const devices = $derived(status?.devices ?? []);

  /** 在线设备是独立浮层，默认关着；停止共享后自己收起来。 */
  let showDevices = $state(false);
  let modePickerOpen = $state(false);

  $effect(() => {
    if (!isSharing || !showSharePanel) showDevices = false;
  });

  const floatingOpen = $derived(showSharePanel || showEntryPicker || showDevices || modePickerOpen);

  function dismissFloating() {
    if (showSharePanel) onCloseSharePanel();
    if (showEntryPicker) onCloseEntryPicker();
    showDevices = false;
    modePickerOpen = false;
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "Escape") return;
    // 先收浮层，浮层都关着才退出沉浸 —— 一次 Esc 只做一件事。
    if (floatingOpen) {
      dismissFloating();
      return;
    }
    if (appLayout.immersive) appLayout.immersive = false;
  }

  function toggleImmersive() {
    dismissFloating();
    appLayout.immersive = !appLayout.immersive;
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<section class="preview-workspace">
  <div class="preview-card">
  <div class="preview-chrome">
    <div class="preview-chrome-left">
      <span class="preview-site-icon"><Icon name={site ? "globe" : "monitor"} size={15} /></span>
      <div class="preview-title-block">
        <strong>{site ? localizeSiteName(site.name) : t("noSiteTitle")}</strong>
        {#if site}
          <small class="preview-title-meta">
            {#if (site.html_entries?.length || 0) > 1}
              <!-- 目录里有多个 HTML 时，入口名本身就是切换入口的按钮。
                   原来这个功能藏在共享面板折叠的检查项里，等于没有。
                   现在点一下展开、再点一下收起 —— 不用每次都点面板上的关闭按钮。 -->
              <button
                type="button"
                class="entry-switch"
                class:open={showEntryPicker}
                onclick={onChooseEntry}
                aria-expanded={showEntryPicker}
                title={showEntryPicker ? t("collapseEntryPicker") : t("switchEntry")}
              >
                <Icon name="file" size={12} /> {site.entry}
                <span class="entry-switch-chevron" class:up={showEntryPicker}><Icon name="chevron" size={11} /></span>
              </button>
            {:else}
              <span><Icon name="file" size={12} /> {site.entry}</span>
            {/if}
            <ModePicker
              {mode}
              proxyPort={status?.proxy_port ?? null}
              servers={devServerList}
              busy={modeBusy}
              bind:open={modePickerOpen}
              onOpen={onScanDevServers}
              {onUseDevServer}
              {onUseStatic}
            />
          </small>
        {/if}
      </div>
    </div>
    <div class="preview-chrome-actions">
      <span class="preview-mode-pill {site ? 'live' : ''} {busy ? 'busy' : ''}">
        {#if busy}<span class="spinner tiny"></span>{:else}<span class="status-dot {isSharing ? 'green' : site ? 'blue' : 'amber'}"></span>{/if}
        {site ? t("liveWatch") : t("notSelected")}
      </span>
      <button class="secondary-button compact" onclick={onChooseFolder} disabled={busy}>
        <Icon name="folder" size={14} /> {site ? t("replace") : t("choose")}
      </button>
      <button class="icon-button" onclick={onRefreshPreview} aria-label={t("refreshPreview")} title={t("refreshPreview")} disabled={!site}>
        <Icon name="refresh" size={15} />
      </button>
      <div class="share-fab-wrap">
        <!-- 这颗按钮只做一件事：开关共享面板。访问者入口在侧边栏，不往这里挤。 -->
        <div class="share-fab {shareButton.tone}" class:open={showSharePanel}>
          <button
            type="button"
            class="share-fab-main"
            onclick={onToggleSharePanel}
            aria-expanded={showSharePanel}
            title={isSharing ? t("openSharePanel") : shareButton.detail}
          >
            <span class="share-fab-icon"><Icon name={isSharing ? "share" : "zap"} size={16} /></span>
            <span class="share-fab-copy">
              <strong>{shareButton.label}</strong>
              {#if !isSharing}
                <small>{shareButton.detail}</small>
              {/if}
            </span>
          </button>
          {#if isSharing}
            <span class="status-dot green"></span>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <div class="preview-stage">
    {#if !site}
      <div class="preview-empty" class:dragging data-dropzone tabindex="0" role="button" aria-label={t("dropSite")}>
        <div class="empty-stage">
          <div class="preview-empty-card has-recent">
            <strong>{t("emptyPreviewTitle")}</strong>
            <p>{t("emptyPreviewHint")}</p>
            <div class="dropzone-actions center">
              <button class="primary-button compact-primary" onclick={onChooseFolder} disabled={busy}>
                <Icon name="folder" size={14} /> {t("chooseFolder")}
              </button>
              <button class="secondary-button" onclick={onChooseHtmlFile} disabled={busy}>{t("chooseHtml")}</button>
            </div>
            <div class="empty-recent-block">
              <div class="empty-recent-head">
                <div><strong>{t("recent")}</strong></div>
                {#if recentCount > 3}
                  <button class="text-button" onclick={onGoRecent}>{t("all")} <Icon name="arrow" size={12} /></button>
                {/if}
              </div>
              {#if emptyRecent.length}
                <div class="empty-recent-list">
                  {#each emptyRecent as item}
                    <button class="empty-recent-item" onclick={() => onReopenRecent(item)}>
                      <span class="recent-file-icon"><Icon name="file" size={15} /></span>
                      <span class="empty-recent-copy">
                        <strong>{localizeSiteName(item.name)}</strong>
                        <small>{localizeUpdated(item.updated)} · {item.entry || "index.html"}</small>
                      </span>
                      <span class="recent-arrow"><Icon name="arrow" size={14} /></span>
                    </button>
                  {/each}
                </div>
              {:else}
                <div class="empty-recent-none">{t("noRecent")}</div>
              {/if}
            </div>
          </div>
          <div class="empty-features">
            <div class="empty-feature">
              <span class="empty-feature-icon"><Icon name="qr" size={15} /></span>
              <strong>{t("featureNoDeploy")}</strong>
              <p>{t("featureNoDeployHint")}</p>
            </div>
            <div class="empty-feature">
              <span class="empty-feature-icon"><Icon name="shield" size={15} /></span>
              <strong>{t("featureAccess")}</strong>
              <p>{t("featureAccessHint")}</p>
            </div>
            <div class="empty-feature">
              <span class="empty-feature-icon"><Icon name="refresh" size={15} /></span>
              <strong>{t("featureLive")}</strong>
              <p>{t("featureLiveHint")}</p>
            </div>
          </div>
        </div>
      </div>
    {:else if !previewUrl}
      <div class="preview-empty static">
        <div class="preview-empty-card">
          <div class="dropzone-icon"><Icon name="monitor" size={22} /></div>
          <strong>{t("loading")}</strong>
          <div class="dropzone-actions center">
            <button class="secondary-button" onclick={onRefreshPreview}><Icon name="refresh" size={14} /> {t("retry")}</button>
            <button class="secondary-button" onclick={onChooseFolder}><Icon name="folder" size={14} /> {t("rechoose")}</button>
          </div>
        </div>
      </div>
    {:else}
      <PreviewFrame {previewUrl} {dragging} />
    {/if}

    {#if floatingOpen && site && previewUrl}
      <!-- iframe 里的点击到不了父页面。面板开着时盖一层透明热区，
           点预览网页任意位置就收起，这一下也不会点进网页里。 -->
      <div class="preview-dismiss" onpointerdown={dismissFloating} aria-hidden="true"></div>
    {/if}

    {#if appLayout.immersive}
      <!-- 沉浸模式里工具条整条收掉，只留右上角的共享和右下角的退出。 -->
      <div class="immersive-share share-fab {shareButton.tone}" class:open={showSharePanel}>
        <button
          type="button"
          class="share-fab-main"
          onclick={onToggleSharePanel}
          aria-expanded={showSharePanel}
          title={isSharing ? t("openSharePanel") : shareButton.detail}
        >
          <span class="share-fab-icon"><Icon name={isSharing ? "share" : "zap"} size={16} /></span>
          <span class="share-fab-copy"><strong>{shareButton.label}</strong></span>
        </button>
        {#if isSharing}
          <span class="status-dot green"></span>
        {/if}
      </div>
      <button type="button" class="immersive-exit" onclick={toggleImmersive} title={t("exitFullscreen")}>
        <Icon name="x" size={14} /> {t("exitFullscreen")}
      </button>
    {/if}

    {#if showEntryPicker}
      <EntryPicker site={site} onClose={onCloseEntryPicker} onSelect={onSelectEntry} />
    {/if}

  </div>

    {#if showSharePanel}
      <SharePanel
        {site}
        {status}
        {isSharing}
        {starting}
        {checking}
        bind:checksExpanded
        {checks}
        {checkItems}
        {qr}
        {previewQr}
        {shareUrl}
        {shareExpiry}
        {shareCustom}
        bind:shareCustomHours
        bind:shareCustomMinutes
        {accessProtection}
        onClose={onCloseSharePanel}
        onToggleChecks={onToggleChecks}
        onRecheck={onRecheck}
        onChooseEntry={onChooseEntry}
        onStart={onStartShare}
        onStop={onStopShare}
        onResetCode={onResetCode}
        onCopyLink={onCopyLink}
        onCopyInvite={onCopyInvite}
        onPreset={onSharePreset}
        onApplyCustom={onApplyShareCustom}
        devicesOpen={showDevices}
        onToggleDevices={() => (showDevices = !showDevices)}
      />
    {/if}

    {#if showDevices && showSharePanel && isSharing}
      <DevicePanel {devices} {tick} onClose={() => (showDevices = false)} {onOpenVisitors} {onBlock} {onBlacklist} />
    {/if}

  <div class="preview-footer-note">
    {#if site}
      <span class="snapshot-inline-tag live"><span class="status-dot green"></span> {t("liveWatch")}</span>
      <button
        type="button"
        class="preview-footer-action"
        onclick={toggleImmersive}
        title={t("fullscreenPreview")}
      >
        <Icon name="monitor" size={13} /> {t("fullscreenPreview")}
      </button>
    {/if}
  </div>
  </div>
</section>

<style>
  .preview-title-meta {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preview-title-meta > span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .entry-switch {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 7px;
    border: 1px solid var(--line-strong);
    border-radius: 999px;
    background: var(--panel);
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .entry-switch:hover {
    border-color: var(--line-strong);
    background: var(--hover);
  }

  .entry-switch.open {
    border-color: var(--blue);
    color: var(--accent-text);
    background: var(--accent-fill);
  }

  .entry-switch-chevron {
    display: inline-flex;
    transition: transform 0.15s ease;
  }

  .entry-switch-chevron.up {
    transform: rotate(180deg);
  }
</style>
