<script setup lang="ts">
import { useElementSize, useEventListener } from "@vueuse/core";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { call } from "../../../api/commands";
import { toAppError } from "../../../api/guards";
import { toastAppError } from "../../../shared/toastAppError";
import { path, report, selection } from "../../store";
import { registerViewerControls } from "../../viewerControls";

const ZOOM_STEP = 1.25;
const MIN_SCALE = 0.05;
const MAX_SCALE = 64;

const frameRef = ref<HTMLElement | null>(null);
const { width: frameW, height: frameH } = useElementSize(frameRef);

const objectUrl = ref<string | null>(null);
const naturalWidth = ref(0);
const naturalHeight = ref(0);
const loading = ref(false);
const loadFailed = ref(false);
const fetchGen = ref(0);

/** User-controlled scale; `null` means fit-to-frame. */
const userScale = ref<number | null>(null);
const panX = ref(0);
const panY = ref(0);

const dragging = ref(false);
const dragOrigin = ref<{ x: number; y: number; panX: number; panY: number } | null>(
  null,
);

const previewUnavailable = computed((): string | null => {
  const preview = report.value?.preview;
  if (preview?.kind === "unavailable") {
    return preview.reason;
  }
  return null;
});

const tonemapped = computed(
  (): boolean => report.value?.preview.kind === "tonemappedFromExr",
);

const fitScale = computed((): number => {
  if (
    naturalWidth.value <= 0 ||
    naturalHeight.value <= 0 ||
    frameW.value <= 0 ||
    frameH.value <= 0
  ) {
    return 1;
  }
  const pad = 8;
  const availW = Math.max(1, frameW.value - pad);
  const availH = Math.max(1, frameH.value - pad);
  return Math.min(availW / naturalWidth.value, availH / naturalHeight.value);
});

const scale = computed((): number => userScale.value ?? fitScale.value);

const sizeLabel = computed((): string | null => {
  if (naturalWidth.value <= 0) {
    return null;
  }
  const pct = Math.round(scale.value * 100);
  return `${String(naturalWidth.value)}×${String(naturalHeight.value)} · ${String(pct)}%`;
});

function revokeUrl(): void {
  if (objectUrl.value !== null) {
    URL.revokeObjectURL(objectUrl.value);
    objectUrl.value = null;
  }
}

function clampScale(value: number): number {
  return Math.min(MAX_SCALE, Math.max(MIN_SCALE, value));
}

function resetPan(): void {
  panX.value = 0;
  panY.value = 0;
}

function zoomFit(): void {
  userScale.value = null;
  resetPan();
}

function zoomActual(): void {
  userScale.value = 1;
  resetPan();
}

function zoomAt(nextScale: number, originX: number, originY: number): void {
  const frame = frameRef.value;
  if (frame === null) {
    userScale.value = clampScale(nextScale);
    return;
  }

  const rect = frame.getBoundingClientRect();
  const cx = originX - rect.left - rect.width / 2;
  const cy = originY - rect.top - rect.height / 2;
  const prev = scale.value;
  const clamped = clampScale(nextScale);
  if (prev <= 0) {
    userScale.value = clamped;
    return;
  }
  const ratio = clamped / prev;
  panX.value = cx - (cx - panX.value) * ratio;
  panY.value = cy - (cy - panY.value) * ratio;
  userScale.value = clamped;
}

function zoomBy(factor: number): void {
  const frame = frameRef.value;
  if (frame === null) {
    userScale.value = clampScale(scale.value * factor);
    return;
  }
  const rect = frame.getBoundingClientRect();
  zoomAt(scale.value * factor, rect.left + rect.width / 2, rect.top + rect.height / 2);
}

function zoomIn(): void {
  zoomBy(ZOOM_STEP);
}

function zoomOut(): void {
  zoomBy(1 / ZOOM_STEP);
}

async function loadImage(): Promise<void> {
  const filePath = path.value;
  const gen = ++fetchGen.value;

  revokeUrl();
  naturalWidth.value = 0;
  naturalHeight.value = 0;
  loadFailed.value = false;
  zoomFit();

  if (filePath === null || filePath === "" || report.value === null) {
    loading.value = false;
    return;
  }

  if (previewUnavailable.value !== null) {
    loading.value = false;
    return;
  }

  loading.value = true;
  try {
    const bytes = await call("inspector_image", {
      path: filePath,
      selection: {
        level: selection.value.level,
        face: selection.value.face,
        layer: selection.value.layer,
        depthSlice: selection.value.depthSlice,
      },
    });
    if (gen !== fetchGen.value) {
      return;
    }
    const url = URL.createObjectURL(
      new Blob([bytes], { type: "image/png" }),
    );
    objectUrl.value = url;
  } catch (err: unknown) {
    if (gen !== fetchGen.value) {
      return;
    }
    loadFailed.value = true;
    toastAppError(toAppError(err));
  } finally {
    if (gen === fetchGen.value) {
      loading.value = false;
    }
  }
}

function onImgLoad(event: Event): void {
  const img = event.target;
  if (!(img instanceof HTMLImageElement)) {
    return;
  }
  naturalWidth.value = img.naturalWidth;
  naturalHeight.value = img.naturalHeight;
}

function onWheel(event: WheelEvent): void {
  if (!event.ctrlKey && !event.metaKey) {
    return;
  }
  event.preventDefault();
  const factor = event.deltaY < 0 ? ZOOM_STEP : 1 / ZOOM_STEP;
  zoomAt(scale.value * factor, event.clientX, event.clientY);
}

function onPointerDown(event: PointerEvent): void {
  if (event.button !== 0 || objectUrl.value === null) {
    return;
  }
  dragging.value = true;
  dragOrigin.value = {
    x: event.clientX,
    y: event.clientY,
    panX: panX.value,
    panY: panY.value,
  };
  frameRef.value?.setPointerCapture(event.pointerId);
}

function onPointerMove(event: PointerEvent): void {
  const origin = dragOrigin.value;
  if (!dragging.value || origin === null) {
    return;
  }
  panX.value = origin.panX + (event.clientX - origin.x);
  panY.value = origin.panY + (event.clientY - origin.y);
}

function onPointerUp(event: PointerEvent): void {
  if (!dragging.value) {
    return;
  }
  dragging.value = false;
  dragOrigin.value = null;
  if (frameRef.value?.hasPointerCapture(event.pointerId) === true) {
    frameRef.value.releasePointerCapture(event.pointerId);
  }
}

watch(
  () =>
    [
      path.value,
      report.value,
      selection.value.level,
      selection.value.face,
      selection.value.layer,
      selection.value.depthSlice,
    ] as const,
  () => {
    void loadImage();
  },
  { immediate: true },
);

useEventListener(frameRef, "wheel", onWheel, { passive: false });

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
  fetchGen.value += 1;
  revokeUrl();
  unregisterControls?.();
});
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-1">
    <div class="flex shrink-0 items-center justify-between gap-2 px-0.5">
      <p
        v-if="tonemapped"
        class="hint m-0"
      >
        Preview, not stored data
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
      :class="dragging ? 'cursor-grabbing' : 'cursor-grab'"
      aria-label="Image preview"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
    >
      <div
        v-if="loading"
        class="absolute inset-0 flex items-center justify-center text-sm text-ink-muted"
      >
        Loading…
      </div>
      <div
        v-else-if="previewUnavailable !== null"
        class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-muted"
      >
        Preview unavailable: {{ previewUnavailable }}
      </div>
      <div
        v-else-if="loadFailed"
        class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-muted"
      >
        Failed to load image
      </div>
      <div
        v-else-if="objectUrl === null"
        class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-faint"
      >
        No image selected
      </div>
      <div
        v-else
        class="absolute inset-0 flex items-center justify-center"
      >
        <div
          class="checkerboard inline-block will-change-transform"
          :style="{
            transform: `translate(${String(panX)}px, ${String(panY)}px) scale(${String(scale)})`,
          }"
        >
          <img
            :src="objectUrl"
            alt="KTX subresource preview"
            class="viewer-img block max-w-none"
            draggable="false"
            @load="onImgLoad"
            @dragstart.prevent
          >
        </div>
      </div>
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
</style>
