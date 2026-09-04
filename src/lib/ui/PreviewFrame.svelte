<script lang="ts">
  import { onMount } from "svelte";
  import { t } from "./i18n";

  const STORAGE_KEY = "vibeshare.previewFrame.v1";
  const MIN_WIDTH = 280;
  const MIN_HEIGHT = 200;

  const PRESETS = [
    { id: "fit", ratio: 0 },
    { id: "16:9", ratio: 16 / 9 },
    { id: "4:3", ratio: 4 / 3 },
    { id: "9:16", ratio: 9 / 16 },
  ] as const;

  type PresetId = (typeof PRESETS)[number]["id"] | "free";

  let { previewUrl, dragging }: { previewUrl: string; dragging: boolean } = $props();

  let stageEl = $state<HTMLDivElement | null>(null);
  let stageWidth = $state(0);
  let stageHeight = $state(0);
  let preset = $state<PresetId>("fit");
  let frameWidth = $state(0);
  let frameHeight = $state(0);
  let resizing = $state(false);
  let restored = false;

  const lockedRatio = $derived(PRESETS.find((item) => item.id === preset)?.ratio ?? 0);
  const sizeLabel = $derived(`${Math.round(preset === "fit" ? stageWidth : frameWidth)} × ${Math.round(preset === "fit" ? stageHeight : frameHeight)}`);

  function clamp(value: number, min: number, max: number) {
    return Math.min(max, Math.max(min, value));
  }

  function inscribed(ratio: number, maxW: number, maxH: number) {
    const widthLimit = Math.max(MIN_WIDTH, maxW);
    const heightLimit = Math.max(MIN_HEIGHT, maxH);
    if (widthLimit / heightLimit > ratio) {
      return { width: heightLimit * ratio, height: heightLimit };
    }
    return { width: widthLimit, height: widthLimit / ratio };
  }

  function persist() {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          preset,
          width: Math.round(frameWidth),
          height: Math.round(frameHeight),
        }),
      );
    } catch {
      // ignore
    }
  }

  function applyPreset(next: PresetId, width = stageWidth, height = stageHeight) {
    preset = next;
    if (next === "fit") {
      frameWidth = width;
      frameHeight = height;
      persist();
      return;
    }
    if (next === "free") {
      persist();
      return;
    }
    const ratio = PRESETS.find((item) => item.id === next)?.ratio || 16 / 9;
    const sized = inscribed(ratio, Math.max(0, width - 24), Math.max(0, height - 24));
    frameWidth = sized.width;
    frameHeight = sized.height;
    persist();
  }

  function constrain(width: number, height: number, ratio: number) {
    let nextWidth = clamp(width, MIN_WIDTH, stageWidth);
    let nextHeight = clamp(height, MIN_HEIGHT, stageHeight);
    if (ratio > 0) {
      if (nextWidth / nextHeight > ratio) nextWidth = nextHeight * ratio;
      else nextHeight = nextWidth / ratio;
      if (nextWidth > stageWidth) {
        nextWidth = stageWidth;
        nextHeight = nextWidth / ratio;
      }
      if (nextHeight > stageHeight) {
        nextHeight = stageHeight;
        nextWidth = nextHeight * ratio;
      }
    }
    return {
      width: clamp(nextWidth, MIN_WIDTH, stageWidth),
      height: clamp(nextHeight, MIN_HEIGHT, stageHeight),
    };
  }

  function startResize(edge: "e" | "s" | "se", event: PointerEvent) {
    event.preventDefault();
    event.stopPropagation();
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);

    let ratio = lockedRatio;
    if (preset === "fit") {
      preset = "free";
      frameWidth = stageWidth;
      frameHeight = stageHeight;
      ratio = 0;
    }

    const startX = event.clientX;
    const startY = event.clientY;
    const startW = frameWidth;
    const startH = frameHeight;
    resizing = true;

    const move = (next: PointerEvent) => {
      let width = startW;
      let height = startH;
      if (edge === "e" || edge === "se") width = startW + (next.clientX - startX);
      if (edge === "s" || edge === "se") height = startH + (next.clientY - startY);
      if (ratio > 0) {
        if (edge === "e") height = width / ratio;
        else if (edge === "s") width = height * ratio;
        else {
          const fromWidth = constrain(width, width / ratio, ratio);
          const fromHeight = constrain(height * ratio, height, ratio);
          const widthDelta = Math.abs(fromWidth.width - startW) + Math.abs(fromWidth.height - startH);
          const heightDelta = Math.abs(fromHeight.width - startW) + Math.abs(fromHeight.height - startH);
          ({ width, height } = widthDelta >= heightDelta ? fromWidth : fromHeight);
          frameWidth = width;
          frameHeight = height;
          return;
        }
      } else {
        preset = "free";
      }
      const nextSize = constrain(width, height, ratio);
      frameWidth = nextSize.width;
      frameHeight = nextSize.height;
    };

    const stop = () => {
      resizing = false;
      persist();
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", stop);
      handle.removeEventListener("pointercancel", stop);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", stop);
    handle.addEventListener("pointercancel", stop);
  }

  function restore(width: number, height: number) {
    try {
      const raw = localStorage.getItem(STORAGE_KEY);
      if (!raw) {
        applyPreset("fit", width, height);
        return;
      }
      const parsed = JSON.parse(raw) as { preset?: PresetId; width?: number; height?: number };
      if (parsed.preset === "16:9" || parsed.preset === "4:3" || parsed.preset === "9:16" || parsed.preset === "fit") {
        applyPreset(parsed.preset, width, height);
        return;
      }
      if (parsed.preset === "free" && parsed.width && parsed.height) {
        preset = "free";
        const next = constrain(parsed.width, parsed.height, 0);
        frameWidth = next.width;
        frameHeight = next.height;
        persist();
      }
    } catch {
      applyPreset("fit", width, height);
    }
  }

  onMount(() => {
    const stage = stageEl;
    if (!stage) return;
    const observer = new ResizeObserver((entries) => {
      const rect = entries[0]?.contentRect;
      if (!rect) return;
      stageWidth = rect.width;
      stageHeight = rect.height;
      if (!restored) {
        restored = true;
        restore(rect.width, rect.height);
        return;
      }
      if (preset === "fit") {
        frameWidth = rect.width;
        frameHeight = rect.height;
      } else if (preset !== "free") {
        applyPreset(preset, rect.width, rect.height);
      } else {
        const next = constrain(frameWidth || rect.width, frameHeight || rect.height, 0);
        frameWidth = next.width;
        frameHeight = next.height;
      }
    });
    observer.observe(stage);
    return () => observer.disconnect();
  });
</script>

<div class="preview-ratio-bar">
  <div class="preview-ratio-group" role="group" aria-label={t("aspectRatio")}>
    {#each PRESETS as item}
      <button
        type="button"
        class="preview-ratio-chip"
        class:active={preset === item.id}
        onclick={() => applyPreset(item.id)}
      >
        {item.id === "fit" ? t("fit") : item.id}
      </button>
    {/each}
    {#if preset === "free"}
      <span class="preview-ratio-chip active">{t("customSize")}</span>
    {/if}
  </div>
  <span class="preview-ratio-size">{sizeLabel}</span>
</div>

<div class="preview-stage-board" bind:this={stageEl} class:resizing class:custom={preset !== "fit"}>
  <div
    class="preview-viewport"
    class:fit={preset === "fit"}
    class:dragging
    data-dropzone
    style:width={preset === "fit" ? "100%" : `${frameWidth}px`}
    style:height={preset === "fit" ? "100%" : `${frameHeight}px`}
  >
    <iframe
      class="preview-frame"
      class:blocked={resizing}
      title={t("sitePreview")}
      src={previewUrl}
      sandbox="allow-scripts allow-same-origin allow-forms allow-modals allow-popups"
    ></iframe>
    <button type="button" class="preview-resize-handle e" aria-label={t("resizeWidth")} onpointerdown={(event) => startResize("e", event)}></button>
    <button type="button" class="preview-resize-handle s" aria-label={t("resizeHeight")} onpointerdown={(event) => startResize("s", event)}></button>
    <button type="button" class="preview-resize-handle se" aria-label={t("resizeAspect")} onpointerdown={(event) => startResize("se", event)}></button>
  </div>
</div>
