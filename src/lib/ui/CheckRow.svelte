<script lang="ts">
  import Icon from "$lib/Icon.svelte";
  import { t } from "./i18n";

  let {
    label,
    detail = "",
    status = "ok",
    action = "",
    onAction,
  }: {
    label: string;
    detail?: string;
    status?: "ok" | "warn";
    action?: string;
    onAction?: () => void;
  } = $props();
</script>

<div class="check-row">
  <span class="check-icon {status}"><Icon name={status === "ok" ? "check" : "alert"} size={16} /></span>
  <span class="check-copy">
    <strong>{label}</strong>
    {#if detail}<small>{detail}</small>{/if}
  </span>
  {#if action === "choose-entry"}
    <button class="text-button" onclick={() => onAction?.()}>{t("chooseEntry")}</button>
  {/if}
</div>
