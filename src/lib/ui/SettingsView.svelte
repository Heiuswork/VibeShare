<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { NetworkInfo } from "$lib/api";
  import { t } from "./i18n";
  import { appLocale, type AppLocale } from "./i18n.svelte";
  import { appTheme, type ThemePreference } from "./theme.svelte";
  import ExpiryPicker from "./ExpiryPicker.svelte";
  import NetworkPicker from "./NetworkPicker.svelte";

  let {
    accessProtection,
    autoStop,
    defaultExpiry,
    defaultCustomHours = $bindable(),
    defaultCustomMinutes = $bindable(),
    network,
    networkBusy = false,
    blacklist = [],
    onSelectNetwork,
    onToggleAccess,
    onToggleAutoStop,
    onSaveDefaultExpiry,
    onRemoveBlacklist,
    onChangeLocale,
    onChangeTheme,
  }: {
    accessProtection: boolean;
    autoStop: boolean;
    defaultExpiry: string;
    defaultCustomHours: number;
    defaultCustomMinutes: number;
    network: NetworkInfo | undefined;
    networkBusy?: boolean;
    blacklist?: string[];
    onSelectNetwork: (ip: string | null) => void;
    onToggleAccess: () => void;
    onToggleAutoStop: () => void;
    onSaveDefaultExpiry: () => void;
    onRemoveBlacklist: (ip: string) => void;
    onChangeLocale: (next: AppLocale) => void;
    onChangeTheme: (next: ThemePreference) => void;
  } = $props();

  let blacklistOpen = $state(false);
</script>

<div class="full-view settings-view">
  <div class="settings-group">
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-copy">
          <strong>{t("settingsAppearance")}</strong>
          <small>{t("settingsAppearanceHint")}</small>
        </div>
        <div class="language-switch" role="group" aria-label={t("settingsAppearance")}>
          <button
            type="button"
            class="language-chip"
            class:active={appTheme.preference === "light"}
            onclick={() => onChangeTheme("light")}
          >
            {t("themeLight")}
          </button>
          <button
            type="button"
            class="language-chip"
            class:active={appTheme.preference === "dark"}
            onclick={() => onChangeTheme("dark")}
          >
            {t("themeDark")}
          </button>
          <button
            type="button"
            class="language-chip"
            class:active={appTheme.preference === "system"}
            onclick={() => onChangeTheme("system")}
          >
            {t("themeSystem")}
          </button>
        </div>
      </div>
      <div class="setting-row">
        <div class="setting-copy">
          <strong>{t("settingsLanguage")}</strong>
          <small>{t("settingsLanguageHint")}</small>
        </div>
        <div class="language-switch" role="group" aria-label={t("settingsLanguage")}>
          <button
            type="button"
            class="language-chip"
            class:active={appLocale.current === "zh"}
            onclick={() => onChangeLocale("zh")}
          >
            {t("languageChinese")}
          </button>
          <button
            type="button"
            class="language-chip"
            class:active={appLocale.current === "en"}
            onclick={() => onChangeLocale("en")}
          >
            {t("languageEnglish")}
          </button>
        </div>
      </div>
    </div>
  </div>
  <div class="settings-group">
    <div class="settings-group-title">{t("settingsShare")}</div>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-copy">
          <strong>{t("settingsAccessCode")}</strong>
          <small>{t("settingsAccessHint")}</small>
        </div>
        <button class="toggle" class:on={accessProtection} onclick={onToggleAccess} aria-label={t("settingsAccessCode")}><span></span></button>
      </div>
      <div class="setting-row">
        <div class="setting-copy">
          <strong>{t("settingsAutoStop")}</strong>
          <small>{t("settingsAutoStopHint")}</small>
        </div>
        <button class="toggle" class:on={autoStop} onclick={onToggleAutoStop} aria-label={t("settingsAutoStop")}><span></span></button>
      </div>
      <div class="setting-row setting-row-stack">
        <div class="setting-copy"><strong>{t("settingsDefaultExpiry")}</strong></div>
        <ExpiryPicker
          compact
          alwaysCustom
          label={t("settingsDefaultExpiry")}
          value={defaultExpiry}
          bind:customHours={defaultCustomHours}
          bind:customMinutes={defaultCustomMinutes}
          applyLabel={t("settingsSave")}
          onApply={onSaveDefaultExpiry}
        />
      </div>
    </div>
  </div>
  <div class="settings-group">
    <div class="settings-group-title">{t("settingsNetwork")}</div>
    <div class="settings-card">
      <div class="setting-row setting-row-stack">
        <div class="setting-copy">
          <strong>{t("settingsShareNetwork")}</strong>
          <small>{t("settingsShareNetworkHint")}</small>
        </div>
        <NetworkPicker {network} busy={networkBusy} onSelect={onSelectNetwork} />
      </div>
      <div class="setting-row setting-row-stack">
        <button
          type="button"
          class="blacklist-toggle"
          class:open={blacklistOpen}
          onclick={() => (blacklistOpen = !blacklistOpen)}
          aria-expanded={blacklistOpen}
        >
          <span class="setting-copy">
            <strong>{t("settingsBlacklist")}</strong>
            <small>{t("settingsBlacklistHint")}</small>
          </span>
          <span class="blacklist-toggle-meta">
            <span class="blacklist-count">{blacklist.length}</span>
            <span class="checks-chevron" class:up={blacklistOpen}><Icon name="chevron" size={14} /></span>
          </span>
        </button>
        {#if blacklistOpen}
          {#if blacklist.length}
            <ul class="blacklist-list">
              {#each blacklist as ip (ip)}
                <li>
                  <span class="blacklist-ip">{ip}</span>
                  <button type="button" class="text-button" onclick={() => onRemoveBlacklist(ip)}>
                    {t("settingsRemove")}
                  </button>
                </li>
              {/each}
            </ul>
          {:else}
            <div class="blacklist-empty">{t("settingsBlacklistEmpty")}</div>
          {/if}
        {/if}
      </div>
    </div>
  </div>
  <div class="settings-group">
    <div class="settings-group-title">{t("settingsAbout")}</div>
    <div class="settings-card">
      <div class="setting-row">
        <div class="setting-copy">
          <strong>VibeShare</strong>
          <small>{t("settingsAboutHint")}</small>
        </div>
        <span class="settings-static-value">0.9.0</span>
      </div>
    </div>
  </div>
</div>

<style>
  .language-switch {
    display: inline-flex;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: 4px;
    padding: 3px;
    border: 1px solid var(--line);
    border-radius: 999px;
    background: var(--inset);
    flex: 0 1 auto;
    max-width: 100%;
  }

  .language-chip {
    min-height: 26px;
    padding: 0 10px;
    border-radius: 999px;
    color: var(--text-label);
    background: transparent;
    font-size: 11px;
    font-weight: 650;
  }

  .language-chip.active {
    color: var(--blue-dark);
    background: var(--raised);
    box-shadow: var(--sh1);
  }

  .blacklist-toggle {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 0;
    background: transparent;
    text-align: left;
  }
  .blacklist-toggle-meta {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    flex: 0 0 auto;
  }

  .blacklist-count {
    min-width: 20px;
    padding: 2px 6px;
    border-radius: 999px;
    background: var(--hover-strong);
    color: var(--muted);
    font-size: 10px;
    font-weight: 700;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .blacklist-list {
    list-style: none;
    margin: 0;
    padding: 0;
    border: 1px solid var(--line);
    border-radius: 8px;
    background: var(--inset);
    overflow: hidden;
  }

  .blacklist-list li {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    min-height: 40px;
    padding: 0 12px;
    border-bottom: 1px solid var(--line-soft);
  }

  .blacklist-list li:last-child {
    border-bottom: 0;
  }

  .blacklist-ip {
    color: var(--text-body);
    font-size: 12px;
    font-variant-numeric: tabular-nums;
  }

  .blacklist-empty {
    padding: 14px 12px;
    border: 1px dashed var(--line);
    border-radius: 8px;
    background: var(--inset);
    color: var(--soft);
    font-size: 11px;
    text-align: center;
  }
</style>
