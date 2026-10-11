<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useZoomPan } from "../useZoomPan";
import { registerViewerControls } from "../viewerControls";

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
  if (comparing.value) {
    // Shared canvas = max of both sides so neither is letterboxed away when sizes differ.
    naturalWidth.value = Math.max(naturalWidth.value, width);
    naturalHeight.value = Math.max(naturalHeight.value, height);
    return;
  }
  if (hasContentSize.value) {
    // Header dims already drive fit; avoid a one-frame 1:1 flash from decode.
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

watch(
  () => [props.contentWidth, props.contentHeight] as const,
  (next, prev) => {
    const nextW = next[0] ?? 0;
    const nextH = next[1] ?? 0;
    const prevW = prev?.[0] ?? 0;
    const prevH = prev?.[1] ?? 0;
    if (nextW > 0 && nextH > 0) {
      applyContentSize();
      if (prevW !== nextW || prevH !== nextH) {
        zoomFit();
      }
      return;
    }
    if (prevW > 0 || prevH > 0) {
      naturalWidth.value = 0;
      naturalHeight.value = 0;
    }
  },
  { immediate: true },
);

watch(
  () => props.src,
  () => {
    if (hasContentSize.value) {
      applyContentSize();
      return;
    }
    // Unknown size: hide until `@load` (fitScale is 1 while natural is 0).
    naturalWidth.value = 0;
    naturalHeight.value = 0;
    zoomFit();
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
  <div class="flex h-full min-h-0 flex-col gap-1">
    <div class="flex shrink-0 items-center justify-between gap-2 px-0.5">
      <p
        v-if="hint"
        class="hint m-0"
      >
        {{ hint }}
      </p>
      <span
        v-else
        class="text-xs text-ink-faint"
      />
      <span
        v-if="sizeLabel !== null"
        class="font-mono text-xs text-ink-faint"
      >
        {{ sizeLabel }}
      </span>
    </div>

    <div
      ref="frameRef"
      class="viewer-frame relative min-h-0 flex-1 overflow-hidden rounded border border-line bg-surface-sunken"
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
          <div class="checkerboard inline-block">
            <img
              :src="src"
              :alt="alt"
              class="viewer-img block max-w-none"
              draggable="false"
              @load="onPrimaryLoad"
              @dragstart.prevent
            >
          </div>
        </div>
      </div>

      <!--
        Compare wipe in *frame* space: both layers share the same transform, but the
        original is revealed through a left strip whose width is splitPct of the frame.
        Pan/zoom move the images; the cut stays under the divisor.
      -->
      <template v-else>
        <div class="absolute inset-0 flex items-center justify-center overflow-hidden">
          <div
            class="will-change-transform"
            :style="transformStyle"
          >
            <div
              class="checkerboard relative"
              :style="canvasStyle"
            >
              <img
                :src="compareSrc"
                :alt="compareAlt"
                class="compare-img"
                draggable="false"
                @load="onCompareLoad"
                @dragstart.prevent
              >
            </div>
          </div>
        </div>
        <div
          class="absolute inset-y-0 left-0 overflow-hidden"
          :style="{ width: `${String(splitPct)}%` }"
        >
          <div
            class="absolute top-0 left-0 flex h-full items-center justify-center"
            :style="revealInnerStyle"
          >
            <div
              class="will-change-transform"
              :style="transformStyle"
            >
              <div
                class="checkerboard relative"
                :style="canvasStyle"
              >
                <img
                  :src="src"
                  :alt="alt"
                  class="compare-img"
                  draggable="false"
                  @load="onPrimaryLoad"
                  @dragstart.prevent
                >
              </div>
            </div>
          </div>
        </div>
      </template>

      <template v-if="comparing && src !== null && !loading && errorLabel === null">
        <div
          class="pointer-events-none absolute inset-y-0 z-10 w-px bg-white/90 shadow-[0_0_0_1px_rgba(0,0,0,0.35)]"
          :style="{ left: `${String(splitPct)}%` }"
        />
        <button
          type="button"
          class="absolute top-1/2 z-20 flex h-10 w-4 -translate-x-1/2 -translate-y-1/2 cursor-col-resize items-center justify-center rounded border border-line bg-surface-raised shadow-sm"
          :style="{ left: `${String(splitPct)}%` }"
          :aria-label="`Compare slider, ${String(Math.round(splitPct))} percent original`"
          @pointerdown="onSplitPointerDown"
          @pointermove="onSplitPointerMove"
          @pointerup="onSplitPointerUp"
          @pointercancel="onSplitPointerUp"
        >
          <span class="h-5 w-0.5 rounded bg-ink-muted" />
        </button>
        <span
          class="pointer-events-none absolute bottom-2 left-2 z-10 rounded bg-black/55 px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-white"
        >
          {{ originalLabel }}
        </span>
        <span
          class="pointer-events-none absolute bottom-2 right-2 z-10 rounded bg-black/55 px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-white"
        >
          {{ resultLabel }}
        </span>
      </template>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.checkerboard {
  background-color: #c0c0c0;
  background-image:
    linear-gradient(45deg, #808080 25%, transparent 25%),
    linear-gradient(-45deg, #808080 25%, transparent 25%),
    linear-gradient(45deg, transparent 75%, #808080 75%),
    linear-gradient(-45deg, transparent 75%, #808080 75%);
  background-size: 16px 16px;
  background-position:
    0 0,
    0 8px,
    8px -8px,
    -8px 0;
}

.viewer-img {
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  width: auto;
  height: auto;
}

.compare-img {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  max-width: none;
  object-fit: fill;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
</style>
