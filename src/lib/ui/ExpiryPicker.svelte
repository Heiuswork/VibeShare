<script lang="ts">
  import { EXPIRY_PRESETS, expiryDisplayLabel, expiryPresetLabel } from "$lib/storage";
  import { t } from "./i18n";

  let {
    compact = false,
    presets = false,
    alwaysCustom = false,
    label,
    value,
    custom = false,
    customHours = $bindable(),
    customMinutes = $bindable(),
    applyLabel,
    onPreset,
    onApply,
  }: {
    compact?: boolean;
    presets?: boolean;
    alwaysCustom?: boolean;
    label: string;
    value: string;
    custom?: boolean;
    customHours: number;
    customMinutes: number;
    applyLabel?: string;
    onPreset?: (value: string) => void;
    onApply?: () => void;
  } = $props();
</script>

<div class="expiry-picker" class:compact>
  <div class="expiry-picker-label">
    <span class="metric-label">{label}</span>
    <strong>{expiryDisplayLabel(value)}</strong>
  </div>
  {#if presets}
    <div class="expiry-preset-row">
      {#each EXPIRY_PRESETS as item}
        <button
          type="button"
          class="expiry-chip"
          class:active={item.value === "custom" ? custom : value === item.value}
          onclick={() => onPreset?.(item.value)}
        >
          {expiryPresetLabel(item.value)}
        </button>
      {/each}
    </div>
  {/if}
  {#if alwaysCustom || custom}
    <div class="expiry-custom-row" class:always={alwaysCustom}>
      <label>{t("hours")} <input type="number" min="0" max="48" step="1" bind:value={customHours} /></label>
      <label>{t("minutes")} <input type="number" min="0" max="59" step="1" bind:value={customMinutes} /></label>
      <button type="button" class="secondary-button compact" onclick={() => onApply?.()}>{applyLabel ?? t("confirm")}</button>
    </div>
  {/if}
</div>
