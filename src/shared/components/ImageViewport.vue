<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useZoomPan } from "../useZoomPan";
import { registerViewerControls } from "../viewerControls";
import ImageCompareLayers from "./ImageCompareLayers.vue";

const props = withDefaults(
  defineProps<{
    /** Primary image (single mode) or original / left side in compare mode. */
    src: string | null;
    /**
     * When set, enables a wipe compare: `src` on the left, this URL on the right.
     * Pan and zoom stay shared.
     */
    compareSrc?: string | null;
    alt?: string;
    compareAlt?: string;
    loading?: boolean;
    emptyLabel?: string;
    errorLabel?: string | null;
    unavailableLabel?: string | null;
    hint?: string | null;
    /** Left wipe label when comparing. */
    originalLabel?: string;
    /** Right wipe label when comparing. */
    resultLabel?: string;
    /**
     * Known pixel size (e.g. KTX header × mip). When set, fit uses these
     * immediately so the image never flashes at 1:1 before `@load`.
     */
    contentWidth?: number | null;
    contentHeight?: number | null;
    /** Flip the bitmap vertically (inspector default; encoder leaves off). */
    flipY?: boolean;
  }>(),
  {
    compareSrc: null,
    alt: "Preview",
    compareAlt: "Compare",
    loading: false,
    emptyLabel: "No image",
    errorLabel: null,
    unavailableLabel: null,
    hint: null,
    originalLabel: "Original",
    resultLabel: "Result",
    contentWidth: null,
    contentHeight: null,
    flipY: false,
  },
);

const comparing = computed(
  (): boolean => props.compareSrc !== null && props.compareSrc !== "",
);

const naturalWidth = ref(0);
const naturalHeight = ref(0);
const splitting = ref(false);
/** Percent of frame width showing the original (left). */
const splitPct = ref(50);

const canInteract = (): boolean =>
  !splitting.value &&
  props.src !== null &&
  props.errorLabel === null &&
  props.unavailableLabel === null;

const {
  frameRef,
  frameW,
  dragging,
  sizeLabel,
  transformStyle,
  zoomIn,
  zoomOut,
  zoomActual,
  zoomFit,
  onPointerDown,
  onPointerMove,
  onPointerUp,
} = useZoomPan(naturalWidth, naturalHeight, canInteract);

const canvasStyle = computed((): { width: string; height: string } | undefined => {
  if (naturalWidth.value <= 0 || naturalHeight.value <= 0) {
    return undefined;
  }
  return {
    width: `${String(naturalWidth.value)}px`,
    height: `${String(naturalHeight.value)}px`,
  };
});

/** Full-frame width for the clipped left layer so its transform matches the right. */
const revealInnerStyle = computed((): { width: string } => ({
  width: `${String(Math.max(frameW.value, 1))}px`,
}));

const hasContentSize = computed(
  (): boolean =>
    props.contentWidth !== null &&
    props.contentWidth > 0 &&
    props.contentHeight !== null &&
    props.contentHeight > 0,
);

const imageSized = computed(
  (): boolean => naturalWidth.value > 0 && naturalHeight.value > 0,
);

function applyContentSize(): void {
  const width = props.contentWidth;
  const height = props.contentHeight;
  if (width === null || width <= 0 || height === null || height <= 0) {
    return;
  }
  naturalWidth.value = width;
  naturalHeight.value = height;
}

function noteNaturalSize(width: number, height: number): void {
  if (width <= 0 || height <= 0) {
    return;
  }
  if (hasContentSize.value) {
    // Header / mip dims own the canvas (Actual size must shrink with mips).
    return;
  }
  if (comparing.value) {
    // Shared canvas = max of both sides so neither is letterboxed away when sizes differ.
    naturalWidth.value = Math.max(naturalWidth.value, width);
    naturalHeight.value = Math.max(naturalHeight.value, height);
    return;
  }
  naturalWidth.value = width;
  naturalHeight.value = height;
}

function onPrimaryLoad(event: Event): void {
  const img = event.target;
  if (!(img instanceof HTMLImageElement)) {
    return;
  }
  noteNaturalSize(img.naturalWidth, img.naturalHeight);
}

function onCompareLoad(event: Event): void {
  const img = event.target;
  if (!(img instanceof HTMLImageElement)) {
    return;
  }
  noteNaturalSize(img.naturalWidth, img.naturalHeight);
}

function dimPair(
  pair: readonly [number | null | undefined, number | null | undefined] | undefined,
): { w: number; h: number } {
  if (pair === undefined) {
    return { w: 0, h: 0 };
  }
  return { w: pair[0] ?? 0, h: pair[1] ?? 0 };
}

function clearNaturalSizeIfHadPrev(prevW: number, prevH: number): void {
  if (prevW > 0 || prevH > 0) {
    naturalWidth.value = 0;
    naturalHeight.value = 0;
  }
}

function syncContentSizeProps(
  next: readonly [number | null | undefined, number | null | undefined],
  prev: readonly [number | null | undefined, number | null | undefined] | undefined,
): void {
  const { w: nextW, h: nextH } = dimPair(next);
  const { w: prevW, h: prevH } = dimPair(prev);
  if (nextW > 0 && nextH > 0) {
    // Update pixel size only — sticky zoom mode (fit / actual / manual) stays.
    applyContentSize();
    return;
  }
  clearNaturalSizeIfHadPrev(prevW, prevH);
}

watch(
  () => [props.contentWidth, props.contentHeight] as const,
  syncContentSizeProps,
  { immediate: true },
);

watch(
  () => props.src,
  () => {
    if (hasContentSize.value) {
      applyContentSize();
      return;
    }
    // Unknown size: hide until `@load`. Keep zoom mode (do not force Fit).
    naturalWidth.value = 0;
    naturalHeight.value = 0;
  },
);

watch(
  () => props.compareSrc,
  (next) => {
    if (next === null || next === "") {
      splitPct.value = 50;
    }
  },
);

function clampSplit(pct: number): number {
  return Math.min(95, Math.max(5, pct));
}

function splitFromClientX(clientX: number): void {
  const frame = frameRef.value;
  if (frame === null) {
    return;
  }
  const rect = frame.getBoundingClientRect();
  if (rect.width <= 0) {
    return;
  }
  splitPct.value = clampSplit(((clientX - rect.left) / rect.width) * 100);
}

function onSplitPointerDown(event: PointerEvent): void {
  if (event.button !== 0 || !comparing.value) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
  splitting.value = true;
  splitFromClientX(event.clientX);
  const handle = event.currentTarget;
  if (handle instanceof HTMLElement) {
    handle.setPointerCapture(event.pointerId);
  }
}

function onSplitPointerMove(event: PointerEvent): void {
  if (!splitting.value) {
    return;
  }
  splitFromClientX(event.clientX);
}

function onSplitPointerUp(event: PointerEvent): void {
  if (!splitting.value) {
    return;
  }
  splitting.value = false;
  const handle = event.currentTarget;
  if (handle instanceof HTMLElement && handle.hasPointerCapture(event.pointerId)) {
    handle.releasePointerCapture(event.pointerId);
  }
}

const frameCursor = computed((): string => {
  if (splitting.value) {
    return "col-resize";
  }
  if (dragging.value) {
    return "grabbing";
  }
  return "grab";
});

let unregisterControls: (() => void) | undefined;

onMounted(() => {
  unregisterControls = registerViewerControls({
    zoomIn,
    zoomOut,
    zoomActual,
    zoomFit,
  });
});

onUnmounted(() => {
  unregisterControls?.();
});
</script>

<template>
  <div
    ref="frameRef"
    class="viewer-frame relative h-full min-h-0 flex-1 overflow-hidden rounded-lg border border-transparent bg-surface-sunken shadow-sm"
    :style="{ cursor: frameCursor }"
    aria-label="Image preview"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <div
      v-if="unavailableLabel !== null"
      class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-muted"
    >
      {{ unavailableLabel }}
    </div>
    <div
      v-else-if="errorLabel !== null"
      class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-muted"
    >
      {{ errorLabel }}
    </div>
    <div
      v-else-if="src === null && loading"
      class="absolute inset-0 flex items-center justify-center text-sm text-ink-muted"
    >
      Loading…
    </div>
    <div
      v-else-if="src === null"
      class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-faint"
    >
      {{ emptyLabel }}
    </div>

    <!-- Single image: transform lives on the image stack -->
    <div
      v-else-if="!comparing || !compareSrc"
      class="absolute inset-0 flex items-center justify-center"
      :class="{ 'opacity-0': !imageSized }"
    >
      <div
        class="will-change-transform"
        :style="transformStyle"
      >
        <div
          class="checkerboard relative inline-block"
          :style="canvasStyle"
        >
          <img
            :src="src"
            :alt="alt"
            class="viewer-img block max-w-none"
            :class="{
              'viewer-img-canvas': canvasStyle !== undefined,
              'scale-y-[-1]': flipY,
            }"
            draggable="false"
            @load="onPrimaryLoad"
            @dragstart.prevent
          >
        </div>
      </div>
    </div>

    <ImageCompareLayers
      v-else-if="compareSrc !== null && compareSrc !== ''"
      :src="src"
      :compare-src="compareSrc"
      :alt="alt"
      :compare-alt="compareAlt"
      :split-pct="splitPct"
      :transform-style="transformStyle"
      :canvas-style="canvasStyle"
      :reveal-inner-style="revealInnerStyle"
      :original-label="originalLabel"
      :result-label="resultLabel"
      :show-chrome="!loading && errorLabel === null"
      :flip-y="flipY"
      @primary-load="onPrimaryLoad"
      @compare-load="onCompareLoad"
      @split-pointer-down="onSplitPointerDown"
      @split-pointer-move="onSplitPointerMove"
      @split-pointer-up="onSplitPointerUp"
    />

    <p
      v-if="hint"
      class="pointer-events-none absolute top-2 left-2 z-10 m-0 max-w-[min(100%-1rem,20rem)] rounded bg-black/55 px-1.5 py-0.5 text-[10px] leading-snug text-white"
    >
      {{ hint }}
    </p>
    <span
      v-if="sizeLabel !== null"
      class="pointer-events-none absolute top-2 right-2 z-10 rounded bg-black/55 px-1.5 py-0.5 font-mono text-[10px] tabular-nums text-white"
    >
      {{ sizeLabel }}
    </span>
  </div>
</template>

<style lang="scss" scoped>
.viewer-img {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  width: auto;
  height: auto;
}

/* Header/mip dims own the box so Actual size + mip scrub match content size. */
.viewer-img-canvas {
  width: 100%;
  height: 100%;
  object-fit: fill;
}
</style>
