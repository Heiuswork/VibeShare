<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { AssetReport } from "$lib/api";
  import CheckRow from "./CheckRow.svelte";
  import { t } from "./i18n";

  let {
    healthy,
    networkReachable,
    networkName,
    networkIp,
    assets,
    isSharing,
    shareUrl,
    port,
    onCopy,
    onRecheck,
  }: {
    healthy: boolean;
    networkReachable: boolean;
    networkName: string;
    networkIp: string;
    assets: AssetReport | null;
    isSharing: boolean;
    shareUrl: string;
    port: number | string;
    onCopy: () => void;
    onRecheck: () => void;
  } = $props();

  const missing = $derived(assets?.missing ?? []);
  const hardcoded = $derived(assets?.hardcoded_local ?? []);
</script>

<div class="full-view diagnostics-view">
  <div class="diagnostic-banner" class:warn={!healthy}>
    <div class="diagnostic-banner-icon"><Icon name="shield" size={20} /></div>
    <div>
      <strong>{healthy ? t("healthy") : t("unreachableByPhone")}</strong>
    </div>
    <div class="diagnostic-banner-actions">
      <button class="secondary-button" onclick={onCopy}><Icon name="copy" size={15} /> {t("copyDiagnostics")}</button>
      <button class="secondary-button" onclick={onRecheck}><Icon name="refresh" size={15} /> {t("recheck")}</button>
    </div>
  </div>
  <div class="diagnostic-columns single">
    <div class="diagnostic-card">
      <div class="section-heading inline"><div><h2>{t("networkAndService")}</h2></div></div>
      <div class="diagnostic-list">
        <CheckRow
          label={t("network")}
          detail={networkReachable ? `${networkName} · ${networkIp}` : t("noNetwork")}
          status={networkReachable ? "ok" : "warn"}
        />
        <CheckRow label={t("firewall")} detail={healthy ? "" : t("allowIncoming")} status="ok" />
        <CheckRow
          label={t("networkIsolation")}
          detail={networkReachable ? "" : t("isolationHint")}
          status={networkReachable ? "ok" : "warn"}
        />
        <CheckRow label={t("shareAddress")} detail={isSharing ? shareUrl : t("notShared")} status="ok" />
        <CheckRow label={t("port")} detail={`${port}${isSharing ? ` · ${t("sharingNow")}` : ""}`} status="ok" />
        <CheckRow
          label={t("staticAssets")}
          detail={assets
            ? missing.length || hardcoded.length
              ? `${missing.length} · ${hardcoded.length}`
              : `${assets.scanned}`
            : t("notScanned")}
          status={assets && (missing.length || hardcoded.length) ? "warn" : "ok"}
        />
      </div>
    </div>
  </div>
  {#if missing.length || hardcoded.length}
    <div class="diagnostic-hints">
      <div class="section-heading inline"><div><h2>{t("resourceIssues")}</h2></div></div>
      <ul>
        {#each missing as item}
          <li>{t("fileMissing")}<code>{item}</code></li>
        {/each}
        {#each hardcoded as item}
          <li>{t("hardcodedTip")}<code>{item}</code></li>
        {/each}
        {#if assets?.truncated}
          <li>{t("moreIssues")}</li>
        {/if}
      </ul>
    </div>
  {/if}
  {#if !healthy}
    <div class="diagnostic-hints">
      <div class="section-heading inline"><div><h2>{t("explain")}</h2></div></div>
      <ul>
        <li>{t("sameNetworkTip")}</li>
        <li>{t("guestNetworkTip")}</li>
      </ul>
    </div>
  {/if}
</div>
