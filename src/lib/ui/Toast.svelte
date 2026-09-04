<script lang="ts">
  /**
   * 提示条。之前不分语气，报错也顶着一个绿色的 ✓ ——
   * "复制失败"和"已开始共享"长得一模一样，看着就很怪。
   */
  let { message, tone = "ok" }: { message: string; tone?: "ok" | "error" } = $props();
</script>

{#if message}
  <div class="toast" class:error={tone === "error"} role="status" aria-live="polite">
    <span class="toast-mark" aria-hidden="true">{tone === "error" ? "!" : "✓"}</span>
    <span>{message}</span>
  </div>
{/if}

<style>
  /* 自己钉死尺寸：这条提示曾被 `#app > *` 的 100%×100% 撑满整个窗口，
     再也不给外面的选择器这个机会。 */
  .toast {
    width: max-content;
    max-width: min(420px, calc(100vw - 48px));
    height: auto;
    flex: 0 0 auto;
  }

  .toast.error {
    background: #4a2530;
  }

  .toast.error .toast-mark {
    color: #ff9d9d;
  }
</style>
