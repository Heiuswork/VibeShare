<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import type { RecentItem } from "$lib/storage";
  import { localizeAssetsMode, localizeSiteName, localizeUpdated, t } from "./i18n";

  let {
    items,
    selectedFolder = "",
    page,
    totalPages,
    total,
    onOpen,
    onAdd,
    onPage,
  }: {
    items: RecentItem[];
    selectedFolder?: string;
    page: number;
    totalPages: number;
    total: number;
    onOpen: (item: RecentItem) => void;
    onAdd: () => void;
    onPage: (page: number) => void;
  } = $props();
</script>

<div class="full-view">
  <div class="view-toolbar">
    <div><h2>{t("pageRecent")}</h2></div>
    <button class="secondary-button" onclick={onAdd}><Icon name="folder" size={16} /> {t("addFile")}</button>
  </div>
  <div class="project-table">
    {#each items as item}
      <button class="project-row" class:selected={selectedFolder === item.folder} onclick={() => onOpen(item)}>
        <span class="project-file-icon"><Icon name="globe" size={18} /></span>
        <span class="project-main">
          <strong>{localizeSiteName(item.name)}</strong>
          <small>{item.folder}</small>
        </span>
        <span class="project-entry"><b>{item.entry}</b><small>{t("entryLabel")}</small></span>
        <span class="project-assets"><b>{localizeAssetsMode(item.assets)}</b><small>{t("assetsLabel")}</small></span>
        <span class="project-updated">{localizeUpdated(item.updated)}</span>
        <span class="project-arrow"><Icon name="arrow" size={16} /></span>
      </button>
    {:else}
      <div class="empty-tip"><Icon name="info" size={15} /> {t("noRecent")}</div>
    {/each}
  </div>
  {#if totalPages > 1}
    <div class="recent-pagination">
      <button class="secondary-button compact" onclick={() => onPage(Math.max(1, page - 1))} disabled={page <= 1}>{t("prevPage")}</button>
      <span>{Math.min(page, totalPages)} / {totalPages} · {total}</span>
      <button class="secondary-button compact" onclick={() => onPage(Math.min(totalPages, page + 1))} disabled={page >= totalPages}>{t("nextPage")}</button>
    </div>
  {/if}
</div>
