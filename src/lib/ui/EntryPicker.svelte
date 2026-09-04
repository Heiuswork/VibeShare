<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { SiteInfo } from "$lib/api";
  import { t } from "./i18n";

  let {
    site,
    onClose,
    onSelect,
  }: {
    site: SiteInfo | null;
    onClose: () => void;
    onSelect: (entry: string) => void;
  } = $props();
</script>

{#if site && (site.html_entries?.length || 0) > 1}
  <div class="entry-picker" data-entry-picker>
    <div class="entry-picker-head">
      <div><strong>{t("chooseEntryTitle")}</strong></div>
      <button class="icon-button" onclick={onClose} aria-label={t("close")}><Icon name="x" size={15} /></button>
    </div>
    <p class="entry-picker-copy">{t("entryPickerHint")}</p>
    <div class="entry-picker-list">
      {#each site.html_entries as entry}
        <button class="entry-option" class:selected={site.entry === entry} onclick={() => onSelect(entry)}>
          <span><Icon name="file" size={14} /></span>
          <span>{entry}</span>
          {#if site.entry === entry}<Icon name="checkSmall" size={14} />{/if}
        </button>
      {/each}
    </div>
  </div>
{/if}
