<script lang="ts">
  import { onMount } from "svelte";
  import "../app.css";
  import { bindLayout } from "$lib/ui/layout.svelte";
  import { bindTheme } from "$lib/ui/theme.svelte";

  let { children } = $props();

  onMount(() => {
    const stopLayout = bindLayout();
    const stopTheme = bindTheme();
    return () => {
      stopLayout();
      stopTheme();
    };
  });
</script>

{@render children()}
