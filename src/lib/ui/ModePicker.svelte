<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { DevServer, ShareMode } from "$lib/api";
  import { t } from "./i18n";

  let {
    mode,
    proxyPort,
    servers,
    busy = false,
    open = $bindable(false),
    onOpen,
    onUseDevServer,
    onUseStatic,
  }: {
    mode: ShareMode;
    proxyPort: number | null;
    servers: DevServer[];
    busy?: boolean;
    open?: boolean;
    /** 展开时才去扫端口 —— 平时没必要每 4 秒探一遍 14 个端口。 */
    onOpen: () => void;
    onUseDevServer: (port: number) => void;
    onUseStatic: () => void;
  } = $props();

  let rootEl = $state<HTMLDivElement | null>(null);

  const label = $derived(mode === "proxy" ? t("devServerPort", { port: proxyPort ?? "?" }) : t("staticFiles"));

  function serverTitle(server: DevServer) {
    const portTitle = server.title.match(/^端口\s*(\d+)$/);
    if (portTitle) return t("localhostPort", { port: portTitle[1] });
    return server.title || t("localhostPort", { port: server.port });
  }

  function toggle(event: MouseEvent) {
    event.stopPropagation();
    open = !open;
    if (open) onOpen();
  }

  $effect(() => {
    if (!open) return;
    function onPointerDown(event: PointerEvent) {
      const target = event.target;
      if (!(target instanceof Node)) return;
      if (rootEl?.contains(target)) return;
      open = false;
    }
    document.addEventListener("pointerdown", onPointerDown, true);
    return () => document.removeEventListener("pointerdown", onPointerDown, true);
  });

  function pick(port: number) {
    open = false;
    onUseDevServer(port);
  }

  function pickStatic() {
    open = false;
    onUseStatic();
  }
</script>

<div class="mode-picker" bind:this={rootEl}>
  <button type="button" class="mode-picker-pill" class:proxy={mode === "proxy"} onclick={toggle} aria-expanded={open}>
    <Icon name={mode === "proxy" ? "server" : "folder"} size={13} />
    <span>{label}</span>
    <span class="checks-chevron" class:up={open}><Icon name="chevron" size={12} /></span>
  </button>
  {#if open}
    <div class="mode-picker-menu" data-share-panel>
      <div class="mode-picker-head">
        <strong>{t("shareContent")}</strong>
        {#if busy}<span class="spinner tiny"></span>{/if}
      </div>
      <button type="button" class="mode-picker-option" class:active={mode === "static"} onclick={pickStatic} disabled={busy}>
        <span class="mode-picker-icon"><Icon name="folder" size={14} /></span>
        <span class="mode-picker-copy">
          <strong>{t("staticFiles")}</strong>
          <small>{t("staticFilesHint")}</small>
        </span>
        {#if mode === "static"}<span class="mode-picker-check"><Icon name="checkSmall" size={14} /></span>{/if}
      </button>
      {#each servers as server (server.port)}
        <button
          type="button"
          class="mode-picker-option"
          class:active={mode === "proxy" && proxyPort === server.port}
          onclick={() => pick(server.port)}
          disabled={busy}
        >
          <span class="mode-picker-icon"><Icon name="server" size={14} /></span>
          <span class="mode-picker-copy">
            <strong>{serverTitle(server)}</strong>
            <small>{t("proxyTo", { port: server.port })}</small>
          </span>
          {#if mode === "proxy" && proxyPort === server.port}
            <span class="mode-picker-check"><Icon name="checkSmall" size={14} /></span>
          {/if}
        </button>
      {:else}
        <button type="button" class="mode-picker-option" disabled title={t("runDevFirst")}>
          <span class="mode-picker-icon"><Icon name="server" size={14} /></span>
          <span class="mode-picker-copy">
            <strong>{t("devServer")}</strong>
            <small>{t("noDevServer")}</small>
          </span>
        </button>
      {/each}
      <p class="mode-picker-hint">
        {t("modePickerHint")}
      </p>
    </div>
  {/if}
</div>

<style>
  .mode-picker {
    position: relative;
  }

  .mode-picker-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 7px;
    border: 1px solid var(--line-strong);
    border-radius: 999px;
    background: var(--panel);
    font-size: 11px;
    color: var(--text-label);
    cursor: pointer;
  }

  .mode-picker-pill.proxy {
    border-color: var(--blue);
    color: var(--accent-text);
    background: var(--accent-fill);
  }

  .mode-picker-menu {
    position: absolute;
    top: calc(100% + 6px);
    left: 0;
    z-index: 40;
    width: 268px;
    display: grid;
    gap: 4px;
    padding: 10px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--panel);
    box-shadow: var(--menu-shadow);
  }

  .mode-picker-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 2px;
    font-size: 12px;
  }

  .mode-picker-option {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 7px 8px;
    border: 1px solid transparent;
    border-radius: 10px;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .mode-picker-option:hover:not(:disabled) {
    background: var(--hover);
  }

  .mode-picker-option:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .mode-picker-option.active {
    border-color: var(--blue);
    background: var(--accent-fill);
  }

  .mode-picker-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.3;
    min-width: 0;
  }

  .mode-picker-copy strong {
    font-size: 12px;
  }

  .mode-picker-copy small {
    font-size: 11px;
    color: var(--muted);
  }

  .mode-picker-check {
    margin-left: auto;
    color: var(--blue);
  }

  .mode-picker-empty {
    padding: 6px 8px;
    font-size: 11px;
    line-height: 1.6;
    color: var(--muted);
  }

  .mode-picker-empty code {
    padding: 1px 4px;
    border-radius: 4px;
    background: var(--hover-strong);
  }

  .mode-picker-hint {
    margin: 4px 0 0;
    padding: 0 2px;
    font-size: 11px;
    line-height: 1.5;
    color: var(--soft);
  }
</style>
