<script lang="ts">
  import BrandMark from "./BrandMark.svelte";

  let { networkName, networkReachable }: { networkName: string; networkReachable: boolean } = $props();
</script>

<header class="titlebar">
  <div class="traffic-lights" aria-hidden="true">
    <span class="traffic red"></span>
    <span class="traffic yellow"></span>
    <span class="traffic green"></span>
  </div>
  <div class="window-title">
    <span class="window-mark"><BrandMark size={20} /></span>
    VibeShare
  </div>
  <div class="titlebar-actions">
    <span class="network-indicator">
      <span class="status-dot {networkReachable ? 'green' : 'amber'}"></span>
      {networkName}
    </span>
  </div>
</header>
