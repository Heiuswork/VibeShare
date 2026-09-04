<script lang="ts">
  import { onMount } from "svelte";
  import {
    assetReport,
    blacklistDevice,
    blacklistedDevices,
    blockDevice,
    devServers,
    openSite,
    pickFolder,
    pickHtmlFile,
    qrPng,
    regenerateCode,
    removeBlacklistedDevice,
    selectNetwork,
    setEntry,
    shareStart,
    shareStatus,
    shareStop,
    useDevServer,
    useStaticFiles,
    type AssetReport,
    type DevServer,
    type ShareStatus,
    type SiteInfo,
  } from "$lib/api";
  import {
    MAX_RECENT,
    RECENT_PAGE_SIZE,
    expiryMinutes,
    formatUpdated,
    isCustomExpiry,
    loadRecent,
    loadSettings,
    normalizeExpiryLabel,
    parseCustomExpiry,
    saveRecent,
    saveSettings,
    type RecentItem,
  } from "$lib/storage";
  import DiagnosticsView from "$lib/ui/DiagnosticsView.svelte";
  import PreviewWorkspace from "$lib/ui/PreviewWorkspace.svelte";
  import RecentView from "$lib/ui/RecentView.svelte";
  import SettingsView from "$lib/ui/SettingsView.svelte";
  import Sidebar from "$lib/ui/Sidebar.svelte";
  import Titlebar from "$lib/ui/Titlebar.svelte";
  import Toast from "$lib/ui/Toast.svelte";
  import VisitorsView from "$lib/ui/VisitorsView.svelte";
  import type { View } from "$lib/ui/types";
  import { appLayout } from "$lib/ui/layout.svelte";
  import { appLocale, setAppLocale } from "$lib/ui/i18n.svelte";
  import { appTheme, setAppTheme } from "$lib/ui/theme.svelte";
  import { localizeError, localizeNetworkLabel, localizeSiteName, t } from "$lib/ui/i18n";

  const savedSettings = loadSettings();

  let view = $state<View>("overview");
  let site = $state<SiteInfo | null>(null);
  let status = $state<ShareStatus | null>(null);
  let toast = $state("");
  let toastTone = $state<"ok" | "error">("ok");
  let qr = $state("");
  let previewQr = $state("");
  let busy = $state(false);
  let starting = $state(false);
  let checking = $state(false);
  let dragging = $state(false);
  let showSharePanel = $state(false);
  let showEntryPicker = $state(false);
  let checksExpanded = $state(false);
  let recentPage = $state(1);
  let previewRevision = $state(0);
  let lastNetworkIp = $state("");
  /** 二维码只在链接变化时重画 —— 状态每 4 秒刷一次，没必要每次都编码一张图。 */
  let qrUrl = $state("");
  /** 让"12 秒前"这类相对时间随秒走，而不是等下一次状态刷新。 */
  let deviceTick = $state(0);
  let assets = $state<AssetReport | null>(null);
  let devServerList = $state<DevServer[]>([]);
  let modeBusy = $state(false);
  let networkBusy = $state(false);
  let blacklist = $state<string[]>([]);

  let accessProtection = $state(savedSettings.accessProtection);
  let autoStop = $state(savedSettings.autoStop);
  let defaultExpiry = $state(savedSettings.defaultExpiry);
  let defaultCustomHours = $state(savedSettings.defaultCustomHours);
  let defaultCustomMinutes = $state(savedSettings.defaultCustomMinutes);
  let shareExpiry = $state(savedSettings.defaultExpiry);
  let shareCustomHours = $state(savedSettings.defaultCustomHours);
  let shareCustomMinutes = $state(savedSettings.defaultCustomMinutes);
  let showShareCustomExpiry = $state(isCustomExpiry(savedSettings.defaultExpiry));
  let recent = $state<RecentItem[]>(loadRecent());
  setAppLocale(savedSettings.locale);
  setAppTheme(savedSettings.theme);

  const network = $derived(status?.network);
  const networkName = $derived.by(() => {
    void appLocale.current;
    if (network?.label) return localizeNetworkLabel(network.label);
    return network?.reachable ? t("localNetwork") : t("noLanDetected");
  });
  const networkIp = $derived(network?.ip || "127.0.0.1");
  const networkReachable = $derived(Boolean(network?.reachable));
  const networkShortLabel = $derived.by(() => {
    void appLocale.current;
    return !networkReachable || networkIp === "127.0.0.1" ? t("thisMacOnly") : t("sameNetwork");
  });
  const isSharing = $derived(Boolean(status?.active));
  const previewUrl = $derived(
    status?.preview_url
      ? `${status.preview_url}${status.preview_url.includes("?") ? "&" : "?"}r=${previewRevision}`
      : "",
  );
  const shareUrl = $derived(status?.url || "");
  const pageTitle = $derived.by(() => {
    void appLocale.current;
    return view === "overview"
      ? t("pagePreview")
      : view === "visitors"
        ? t("pageVisitors")
        : view === "recent"
          ? t("pageRecent")
          : view === "diagnostics"
            ? t("pageDiagnostics")
            : t("pageSettings");
  });
  const devices = $derived(status?.devices ?? []);
  const shareCustom = $derived(isCustomExpiry(shareExpiry) || showShareCustomExpiry);
  const assetDetail = $derived.by(() => {
    void appLocale.current;
    if (!site) return "";
    if (status?.mode === "proxy") return t("servedByDev");
    if (!assets) return t("scanning");
    const missing = assets.missing.length;
    const hardcoded = assets.hardcoded_local.length;
    if (!missing && !hardcoded) return t("scannedRefs", { n: assets.scanned });
    const parts: string[] = [];
    if (missing) parts.push(t("nMissing", { n: missing, file: assets.missing[0] }));
    if (hardcoded) parts.push(t("nHardcoded", { n: hardcoded, file: assets.hardcoded_local[0] }));
    return parts.join(" · ");
  });
  const assetsOk = $derived(
    !site ||
      status?.mode === "proxy" ||
      !assets ||
      (!assets.missing.length && !assets.hardcoded_local.length),
  );
  const checkItems = $derived.by(() => {
    void appLocale.current;
    const networkOk = networkReachable;
    const networkDetail = !networkReachable ? t("noNetwork") : `${networkName} · ${networkIp}`;
    return [
      {
        id: "entry",
        label: t("entryFile"),
        detail: site?.entry || t("noEntry"),
        status: site?.entry ? "ok" as const : "warn" as const,
        action: (site?.html_entries?.length || 0) > 1 ? "choose-entry" : "",
      },
      {
        id: "assets",
        label: t("staticAssets"),
        detail: assetDetail,
        status: site && assetsOk ? "ok" as const : "warn" as const,
      },
      {
        id: "network",
        label: t("network"),
        detail: networkDetail,
        status: networkOk ? "ok" as const : "warn" as const,
      },
    ];
  });
  const checks = $derived.by(() => {
    void appLocale.current;
    const okCount = checkItems.filter((item) => item.status === "ok").length;
    const warnCount = checkItems.length - okCount;
    if (checking) return { label: t("checking"), tone: "checking" as const, okCount, warnCount, total: checkItems.length };
    if (!warnCount) return { label: t("checkOk"), tone: "ok" as const, okCount, warnCount, total: checkItems.length };
    return { label: t("nIssues", { n: warnCount }), tone: "warn" as const, okCount, warnCount, total: checkItems.length };
  });
  const shareButton = $derived.by(() => {
    void appLocale.current;
    if (starting) return { tone: "starting" as const, label: t("preparing"), detail: t("startingShare") };
    if (isSharing) return { tone: "live" as const, label: t("sharingLive"), detail: t("nDevices", { n: status?.connections || 0 }) };
    if (site) return { tone: "ready" as const, label: t("share"), detail: t("shareReady") };
    return { tone: "idle" as const, label: t("share"), detail: t("chooseSiteFirst") };
  });
  const recentTotalPages = $derived(Math.max(1, Math.ceil(recent.length / RECENT_PAGE_SIZE)));
  const recentPageItems = $derived.by(() => {
    const page = Math.min(Math.max(1, recentPage), recentTotalPages);
    const start = (page - 1) * RECENT_PAGE_SIZE;
    return recent.slice(start, start + RECENT_PAGE_SIZE);
  });
  const diagnosticHealthy = $derived(networkReachable && assetsOk);
  const emptyRecent = $derived(recent.slice(0, 3));

  function persistSettings() {
    saveSettings({
      accessProtection,
      autoStop,
      defaultExpiry,
      defaultCustomHours,
      defaultCustomMinutes,
      locale: appLocale.current,
      theme: appTheme.preference,
    });
  }

  function showToast(message: string, tone: "ok" | "error" = "ok") {
    toast = message;
    toastTone = tone;
    window.setTimeout(() => {
      if (toast === message) toast = "";
    }, 2200);
  }

  /** 报错统一走这里，省得每处都记着传 tone。 */
  function showError(error: unknown) {
    showToast(errorMessage(error), "error");
  }

  function isCancelled(error: unknown) {
    return String(error).includes("已取消") || String(error).includes("Cancelled");
  }

  function errorMessage(error: unknown) {
    return localizeError(error);
  }

  function pushRecent(info: SiteInfo) {
    const next: RecentItem = {
      name: info.name,
      folder: info.root,
      entry: info.entry,
      assets: "static",
      updated: formatUpdated(),
      htmlEntries: info.html_entries,
    };
    recent = [next, ...recent.filter((item) => item.folder !== next.folder)].slice(0, MAX_RECENT);
    saveRecent(recent);
  }

  async function refreshStatus(options: { toastNetwork?: boolean } = {}) {
    const next = await shareStatus();
    const previousIp = lastNetworkIp;
    status = next;
    if (next.network.ip && next.network.ip !== "127.0.0.1") lastNetworkIp = next.network.ip;
    const url = next.active ? next.url || "" : "";
    // 只有链接真的变了才重新编码二维码。
    if (url !== qrUrl) {
      qrUrl = url;
      qr = url ? await qrPng(url) : "";
    }
    if (
      options.toastNetwork &&
      previousIp &&
      next.network.ip &&
      previousIp !== next.network.ip &&
      next.network.ip !== "127.0.0.1"
    ) {
      showToast(t("networkChanged", { from: previousIp, to: next.network.ip }));
    }
    return next;
  }

  /** 资源扫描是磁盘 IO，只在站点/入口/模式变化和手动重查时跑，不挂在轮询上。 */
  async function scanAssets() {
    if (!site) {
      assets = null;
      return;
    }
    try {
      assets = await assetReport();
    } catch {
      assets = null;
    }
  }

  /**
   * 选完网站不再自作主张弹入口选择器 —— 用户只是想选个目录，
   * 面板自己蹦出来还配一条 toast，太横。入口有歧义时靠检查项里的
   * "切换入口"按钮提示，什么时候看由用户决定。
   */
  async function applySite(info: SiteInfo) {
    site = info;
    view = "overview";
    showEntryPicker = false;
    previewRevision += 1;
    pushRecent(info);
    await refreshStatus();
    await scanAssets();
    const ambiguous = (info.html_entries?.length || 0) > 1;
    showToast(
      ambiguous
        ? t("selectedSiteEntry", { name: localizeSiteName(info.name), entry: info.entry })
        : t("selectedSite", { name: localizeSiteName(info.name) }),
    );
  }

  async function runSiteAction(action: () => Promise<SiteInfo>) {
    try {
      busy = true;
      checking = true;
      await applySite(await action());
    } catch (error) {
      if (!isCancelled(error)) showError(error);
    } finally {
      busy = false;
      checking = false;
    }
  }

  function chooseFolder() {
    return runSiteAction(pickFolder);
  }

  function chooseHtmlFile() {
    return runSiteAction(pickHtmlFile);
  }

  function reopenRecent(item: RecentItem) {
    if (!item.folder) {
      showToast(t("cannotOpenProject"), "error");
      return;
    }
    return runSiteAction(() => openSite(item.folder, item.entry));
  }

  function chooseDroppedPath(path: string) {
    const normalized = path.replaceAll("\\", "/");
    if (/\.html?$/i.test(normalized)) {
      const index = normalized.lastIndexOf("/");
      return runSiteAction(() => openSite(normalized.slice(0, index), normalized.slice(index + 1)));
    }
    return runSiteAction(() => openSite(normalized));
  }

  async function startShare() {
    if (!site) {
      showToast(t("chooseSiteFirst"), "error");
      return;
    }
    try {
      starting = true;
      // 不强开面板：点这颗按钮的人本来就在面板里。从别处触发的调用方
      // 自己负责先把面板打开（见 startShareFromVisitors）。
      status = await shareStart({
        expiryMinutes: expiryMinutes(shareExpiry, autoStop),
        accessProtection,
      });
      qrUrl = status.url || "";
      qr = qrUrl ? await qrPng(qrUrl) : "";
      showToast(t("shareStarted"));
    } catch (error) {
      showError(error);
    } finally {
      starting = false;
    }
  }

  /** 访问者页的空状态里点"开始共享"：先回到预览页，共享面板在那里。 */
  function startShareFromVisitors() {
    view = "overview";
    showSharePanel = true;
    return startShare();
  }

  async function stopShare() {
    status = await shareStop();
    qrUrl = "";
    qr = "";
    showToast(t("shareStopped"));
  }

  async function blockVisitor(ip: string) {
    try {
      status = await blockDevice(ip);
      showToast(t("blockedIp", { ip }));
    } catch (error) {
      showError(error);
    }
  }

  async function blacklistVisitor(ip: string) {
    try {
      status = await blacklistDevice(ip);
      blacklist = await blacklistedDevices();
      showToast(t("blacklistedIp", { ip }));
    } catch (error) {
      showError(error);
    }
  }

  async function unblacklistVisitor(ip: string) {
    try {
      blacklist = await removeBlacklistedDevice(ip);
      showToast(t("unblacklistedIp", { ip }));
    } catch (error) {
      showError(error);
    }
  }

  async function resetCode() {
    try {
      status = await regenerateCode();
      qrUrl = status.url || "";
      qr = qrUrl ? await qrPng(qrUrl) : "";
      showToast(t("codeReset"));
    } catch (error) {
      showError(error);
    }
  }

  async function scanDevServers() {
    try {
      modeBusy = true;
      devServerList = await devServers();
    } catch (error) {
      showError(error);
    } finally {
      modeBusy = false;
    }
  }

  async function switchToDevServer(port: number) {
    try {
      modeBusy = true;
      status = await useDevServer(port);
      previewRevision += 1;
      assets = null;
      showToast(t("proxiedTo", { port }));
    } catch (error) {
      showError(error);
    } finally {
      modeBusy = false;
    }
  }

  async function switchToStatic() {
    try {
      modeBusy = true;
      status = await useStaticFiles();
      previewRevision += 1;
      await scanAssets();
      showToast(t("switchedToStatic"));
    } catch (error) {
      showError(error);
    } finally {
      modeBusy = false;
    }
  }

  async function chooseNetwork(ip: string | null) {
    try {
      networkBusy = true;
      status = await selectNetwork(ip);
      // 换网卡意味着换地址，链接和二维码都得跟着走。
      qrUrl = status.active ? status.url || "" : "";
      qr = qrUrl ? await qrPng(qrUrl) : "";
      if (status.network.ip) lastNetworkIp = status.network.ip;
      showToast(t("switchedToNetwork", { name: localizeNetworkLabel(status.network.label) || status.network.ip }));
    } catch (error) {
      showError(error);
    } finally {
      networkBusy = false;
    }
  }

  async function copyText(text: string, message: string) {
    if (!text) {
      showToast(t("nothingToCopy"), "error");
      return;
    }
    try {
      await navigator.clipboard.writeText(text);
      showToast(message);
    } catch {
      showToast(t("copyFailed"), "error");
    }
  }

  function copyInvite() {
    if (!shareUrl) return copyText("", "");
    const text = `${t("inviteOpen", { url: shareUrl })}\n${status?.access_code ? t("inviteCode", { code: status.access_code }) : t("inviteNoCode")}`;
    return copyText(text, t("inviteCopied"));
  }

  function copyDiagnostics() {
    const text = [
      t("diagNetwork", { name: networkName, ip: networkIp }),
      t("diagReachable", { value: networkReachable ? t("yes") : t("no") }),
      t("diagEntry", { value: site?.entry || t("notSelected") }),
      t("diagRoot", { value: site?.root || t("notSelected") }),
      t("diagShare", { value: isSharing ? shareUrl : t("notShared") }),
      t("diagPort", { value: status?.port || "—" }),
      t("diagMode", { value: status?.mode === "proxy" ? t("diagModeProxy", { port: status?.proxy_port ?? "?" }) : t("staticFiles") }),
      t("diagAssets", { value: assetDetail || t("notScanned") }),
    ].join("\n");
    return copyText(text, t("diagnosticsCopied"));
  }

  async function recheck() {
    checking = true;
    try {
      await refreshStatus({ toastNetwork: true });
      await scanAssets();
      showToast(t("rechecked"));
    } finally {
      checking = false;
    }
  }

  async function selectEntry(entry: string) {
    try {
      site = await setEntry(entry);
      showEntryPicker = false;
      previewRevision += 1;
      pushRecent(site);
      await refreshStatus();
      await scanAssets();
      showToast(t("entrySwitched", { entry }));
    } catch (error) {
      showError(error);
    }
  }

  function refreshPreview() {
    previewRevision += 1;
    refreshStatus().catch(() => {});
  }

  function applySharePreset(value: string) {
    if (value === "custom") {
      showShareCustomExpiry = true;
      const parsed = parseCustomExpiry(shareExpiry);
      shareCustomHours = parsed.hours;
      shareCustomMinutes = parsed.minutes;
      return;
    }
    showShareCustomExpiry = false;
    shareExpiry = value;
  }

  function applyShareCustomExpiry() {
    shareExpiry = normalizeExpiryLabel("custom", shareCustomHours, shareCustomMinutes);
    showShareCustomExpiry = true;
  }

  function applyDefaultCustomExpiry() {
    defaultExpiry = normalizeExpiryLabel("custom", defaultCustomHours, defaultCustomMinutes);
    shareExpiry = defaultExpiry;
    shareCustomHours = defaultCustomHours;
    shareCustomMinutes = defaultCustomMinutes;
    persistSettings();
    showToast(t("defaultExpirySaved"));
  }

  onMount(() => {
    document.documentElement.classList.add("native-shell");
    qrPng("VibeShare preview").then((value) => (previewQr = value)).catch(() => {});
    refreshStatus().catch(() => {});
    blacklistedDevices().then((entries) => (blacklist = entries)).catch(() => {});
    const timer = window.setInterval(() => {
      refreshStatus({ toastNetwork: true }).catch(() => {});
    }, 4000);
    // 相对时间自己走秒；页面刷新由访问页轮询 /__vibeshare/revision 负责，
    // 后端不再发 site-changed 事件，所以这里也不监听了。
    const tickTimer = window.setInterval(() => {
      deviceTick += 1;
    }, 1000);
    let stopDrag: (() => void) | undefined;
    import("@tauri-apps/api/webview")
      .then(async ({ getCurrentWebview }) => {
        stopDrag = await getCurrentWebview().onDragDropEvent((event) => {
          if (event.payload.type === "over") dragging = true;
          else if (event.payload.type === "leave") dragging = false;
          else if (event.payload.type === "drop") {
            dragging = false;
            const path = event.payload.paths[0];
            if (path) chooseDroppedPath(path);
          }
        });
      })
      .catch(() => {});
    return () => {
      window.clearInterval(timer);
      window.clearInterval(tickTimer);
      stopDrag?.();
    };
  });
</script>

<div class="window" data-sharing={isSharing} data-layout={appLayout.mode} class:immersive={appLayout.immersive} style="--ui-scale: {appLayout.scale}">
  <Titlebar {networkName} {networkReachable} />

  <div class="app-body">
    <Sidebar
      {view}
      recentCount={recent.length}
      visitorCount={devices.length}
      {isSharing}
      {networkName}
      {networkIp}
      {networkReachable}
      {networkShortLabel}
      onView={(next) => (view = next)}
    />

    <main class="main-content" class:preview-main={view === "overview"}>
      {#if view !== "overview"}
        <div class="content-header">
          <div>
            <h1>{pageTitle}</h1>
          </div>
          <div class="header-actions">
            {#if isSharing}
              <!-- 纯展示：数量看这里，要看是谁走侧边栏"访问者"。 -->
              <span class="live-pill">
                <span class="status-dot green"></span> {t("devicesOnline", { n: devices.length })}
              </span>
            {/if}
          </div>
        </div>
      {/if}

      {#if view === "overview"}
        <PreviewWorkspace
          {site}
          {status}
          tick={deviceTick}
          {devServerList}
          {modeBusy}
          {busy}
          {starting}
          {checking}
          {dragging}
          {previewUrl}
          {previewQr}
          {qr}
          {shareUrl}
          {showSharePanel}
          {showEntryPicker}
          bind:checksExpanded
          {shareButton}
          {checks}
          {checkItems}
          {shareExpiry}
          {shareCustom}
          bind:shareCustomHours
          bind:shareCustomMinutes
          {emptyRecent}
          recentCount={recent.length}
          {isSharing}
          {accessProtection}
          onChooseFolder={chooseFolder}
          onChooseHtmlFile={chooseHtmlFile}
          onRefreshPreview={refreshPreview}
          onToggleSharePanel={() => (showSharePanel = !showSharePanel)}
          onReopenRecent={reopenRecent}
          onGoRecent={() => (view = "recent")}
          onSelectEntry={selectEntry}
          onCloseEntryPicker={() => (showEntryPicker = false)}
          onCloseSharePanel={() => (showSharePanel = false)}
          onToggleChecks={() => (checksExpanded = !checksExpanded)}
          onRecheck={recheck}
          onChooseEntry={() => (showEntryPicker = !showEntryPicker)}
          onStartShare={startShare}
          onStopShare={stopShare}
          onResetCode={resetCode}
          onCopyLink={() => copyText(shareUrl, t("urlCopied"))}
          onCopyInvite={copyInvite}
          onSharePreset={applySharePreset}
          onApplyShareCustom={applyShareCustomExpiry}
          onScanDevServers={scanDevServers}
          onUseDevServer={switchToDevServer}
          onUseStatic={switchToStatic}
          onOpenVisitors={() => (view = "visitors")}
          onBlock={blockVisitor}
          onBlacklist={blacklistVisitor}
        />
      {:else if view === "visitors"}
        <VisitorsView
          {devices}
          tick={deviceTick}
          {isSharing}
          {shareUrl}
          accessCode={status?.access_code || ""}
          expiresAt={status?.expires_at ?? null}
          onStartShare={startShareFromVisitors}
          onCopyInvite={copyInvite}
          onBlock={blockVisitor}
          onBlacklist={blacklistVisitor}
        />
      {:else if view === "recent"}
        <RecentView
          items={recentPageItems}
          selectedFolder={site?.root || ""}
          page={recentPage}
          totalPages={recentTotalPages}
          total={recent.length}
          onOpen={reopenRecent}
          onAdd={chooseFolder}
          onPage={(page) => (recentPage = page)}
        />
      {:else if view === "diagnostics"}
        <DiagnosticsView
          healthy={diagnosticHealthy}
          {networkReachable}
          {networkName}
          {networkIp}
          {assets}
          {isSharing}
          {shareUrl}
          port={status?.port || 4173}
          onCopy={copyDiagnostics}
          onRecheck={recheck}
        />
      {:else}
        <SettingsView
          {accessProtection}
          {autoStop}
          {defaultExpiry}
          bind:defaultCustomHours
          bind:defaultCustomMinutes
          {network}
          {networkBusy}
          {blacklist}
          onSelectNetwork={chooseNetwork}
          onToggleAccess={() => {
            accessProtection = !accessProtection;
            persistSettings();
          }}
          onToggleAutoStop={() => {
            autoStop = !autoStop;
            persistSettings();
          }}
          onSaveDefaultExpiry={applyDefaultCustomExpiry}
          onRemoveBlacklist={unblacklistVisitor}
          onChangeLocale={(next) => {
            setAppLocale(next);
            persistSettings();
          }}
          onChangeTheme={(next) => {
            setAppTheme(next);
            persistSettings();
          }}
        />
      {/if}
    </main>
  </div>
</div>

<Toast message={toast} tone={toastTone} />
