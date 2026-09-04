<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "./i18n";

  let {
    deviceName,
    onBlock,
    onBlacklist,
  }: {
    deviceName: string;
    onBlock: () => void;
    onBlacklist: () => void;
  } = $props();

  let open = $state(false);
  let rootEl = $state<HTMLDivElement | null>(null);

  function choose(action: () => void) {
    open = false;
    action();
  }

  $effect(() => {
    if (!open) return;
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (target instanceof Node && !rootEl?.contains(target)) open = false;
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    return () => document.removeEventListener("pointerdown", onPointerDown, true);
  });
</script>

<div class="device-actions" bind:this={rootEl}>
  <button
    type="button"
    class="icon-button device-actions-trigger"
    class:open
    onclick={() => (open = !open)}
    aria-label={`${t("manageDevice")} ${deviceName}`}
    aria-expanded={open}
    title={t("manageDevice")}
  >
    <Icon name="more" size={17} />
  </button>
  {#if open}
    <div class="device-actions-menu" role="menu">
      <button type="button" role="menuitem" onclick={() => choose(onBlock)}>
        <span>{t("actionBlock")}</span>
        <small>{t("actionBlockHint")}</small>
      </button>
      <button type="button" role="menuitem" class="danger" onclick={() => choose(onBlacklist)}>
        <span>{t("actionBlacklist")}</span>
        <small>{t("actionBlacklistHint")}</small>
      </button>
    </div>
  {/if}
</div>

<style>
  .device-actions { position: relative; flex: 0 0 auto; }
  .device-actions-trigger { width: 28px; height: 28px; }
  .device-actions-trigger.open { color: var(--blue-dark); background: var(--blue-soft); }
  .device-actions-menu {
    position: absolute;
    z-index: 30;
    top: calc(100% + 5px);
    right: 0;
    width: 178px;
    padding: 5px;
    border: 1px solid var(--line-strong);
    border-radius: 10px;
    background: var(--panel);
    box-shadow: var(--menu-shadow);
  }
  .device-actions-menu button {
    display: grid;
    width: 100%;
    gap: 2px;
    padding: 8px 9px;
    border-radius: 7px;
    background: transparent;
    color: var(--text-body);
    text-align: left;
  }
  .device-actions-menu button:hover { background: var(--hover); }
  .device-actions-menu span { font-size: 11px; font-weight: 650; }
  .device-actions-menu small { color: var(--muted); font-size: 10px; }
  .device-actions-menu button.danger { color: var(--danger-text); }
  .device-actions-menu button.danger:hover { background: var(--danger-fill); }
  .device-actions-menu button.danger small { color: var(--red); }
</style>
