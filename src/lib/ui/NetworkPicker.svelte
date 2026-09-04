<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { NetworkInfo } from "$lib/api";
  import { localizeNetworkLabel, t } from "./i18n";

  let {
    network,
    busy = false,
    onSelect,
  }: {
    network: NetworkInfo | undefined;
    busy?: boolean;
    onSelect: (ip: string | null) => void;
  } = $props();

  const interfaces = $derived(network?.interfaces ?? []);
  const current = $derived(network?.ip ?? "");
  // 只有一张网卡时选择器没有意义 —— 多网卡（Wi-Fi + 以太网 + 虚拟机网卡）
  // 才是"手机连的是 Wi-Fi，但链接给的是另一张网卡地址"这个问题的来源。
  const multiple = $derived(interfaces.length > 1);
</script>

<div class="network-picker">
  {#if multiple}
    <div class="network-picker-list">
      {#each interfaces as item (item.ip)}
        <button
          type="button"
          class="network-option"
          class:active={item.ip === current}
          disabled={busy}
          onclick={() => onSelect(item.ip)}
        >
          <span class="network-option-icon"><Icon name="wifi" size={14} /></span>
          <span class="network-option-copy">
            <strong>{localizeNetworkLabel(item.label)}</strong>
            <small>{item.ip}</small>
          </span>
          {#if item.ip === current}<span class="network-option-check"><Icon name="checkSmall" size={14} /></span>{/if}
        </button>
      {/each}
    </div>
    <p class="network-picker-hint">
      {t("networkPickerHint")}
    </p>
  {:else}
    <div class="network-picker-single">
      <Icon name="wifi" size={14} />
      <span>{network ? localizeNetworkLabel(network.label) : t("noLanDetected")}{current ? ` · ${current}` : ""}</span>
    </div>
  {/if}
</div>

<style>
  .network-picker-list {
    display: grid;
    gap: 6px;
  }

  .network-option {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 8px 10px;
    border: 1px solid var(--line-strong);
    border-radius: 10px;
    background: var(--panel);
    text-align: left;
    cursor: pointer;
  }

  .network-option:disabled {
    opacity: 0.6;
    cursor: default;
  }

  .network-option.active {
    border-color: var(--blue);
    background: var(--accent-fill);
  }

  .network-option-copy {
    display: flex;
    flex-direction: column;
    line-height: 1.25;
  }

  .network-option-copy small {
    color: var(--muted);
    font-size: 11px;
  }

  .network-option-check {
    margin-left: auto;
    color: var(--blue);
  }

  .network-picker-hint {
    margin: 8px 0 0;
    font-size: 11px;
    line-height: 1.5;
    color: var(--muted);
  }

  .network-picker-single {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-label);
  }
</style>
