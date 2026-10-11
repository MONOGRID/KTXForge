<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { toAppError } from "../../../api/guards";
import type { ImageSelection } from "../../../bindings/ImageSelection";
import ImageViewport from "../../../shared/components/ImageViewport.vue";
import { toastAppError } from "../../../shared/toastAppError";
import {
  clearInspectorImageCache,
  loadInspectorImageUrl,
  peekInspectorImageUrl,
  prefetchInspectorNeighbors,
  warmInspectorMipChain,
  type SubresourceDims,
} from "../../imageCache";
import { path, report, selection } from "../../store";

const MISS_DEBOUNCE_MS = 120;
const PREFETCH_IDLE_MS = 200;

const objectUrl = ref<string | null>(null);
const loading = ref(false);
const loadFailed = ref(false);
const fetchGen = ref(0);
const cachedPath = ref<string | null>(null);

let missTimer: ReturnType<typeof setTimeout> | null = null;
let prefetchTimer: ReturnType<typeof setTimeout> | null = null;

const previewUnavailable = computed((): string | null => {
  const preview = report.value?.preview;
  if (preview?.kind === "unavailable") {
    return preview.reason;
  }
  return null;
});

const tonemapHint = computed((): string | null =>
  report.value?.preview.kind === "tonemappedFromExr"
    ? "Preview, not stored data"
    : null,
);

const unavailableLabel = computed((): string | null => {
  if (previewUnavailable.value === null) {
    return null;
  }
  return `Preview unavailable: ${previewUnavailable.value}`;
});

const errorLabel = computed((): string | null =>
  loadFailed.value ? "Failed to load image" : null,
);

const subresourceDims = computed((): SubresourceDims | null => {
  const current = report.value;
  if (current === null) {
    return null;
  }
  if (current.kind.kind === "ktx2") {
    const header = current.kind.info.header;
    return {
      levelCount: Math.max(1, header.levelCount),
      faceCount: Math.max(1, header.faceCount),
      layerCount: Math.max(1, header.layerCount),
      depthCount: Math.max(1, header.pixelDepth),
    };
  }
  const header = current.kind.header;
  return {
    levelCount: Math.max(1, header.numberOfMipmapLevels),
    faceCount: Math.max(1, header.numberOfFaces),
    layerCount: Math.max(1, header.numberOfArrayElements),
    depthCount: Math.max(1, header.pixelDepth),
  };
});

function mipSize(base: number, level: number): number {
  return Math.max(1, base >> level);
}

/** Header dims for the current mip — drives fit before the PNG decode finishes. */
const contentWidth = computed((): number | null => {
  const current = report.value;
  if (current === null) {
    return null;
  }
  const level = selection.value.level;
  if (current.kind.kind === "ktx2") {
    return mipSize(current.kind.info.header.pixelWidth, level);
  }
  return mipSize(current.kind.header.pixelWidth, level);
});

const contentHeight = computed((): number | null => {
  const current = report.value;
  if (current === null) {
    return null;
  }
  const level = selection.value.level;
  if (current.kind.kind === "ktx2") {
    return mipSize(current.kind.info.header.pixelHeight, level);
  }
  return mipSize(current.kind.header.pixelHeight, level);
});

function isAbortError(err: unknown): boolean {
  return err instanceof DOMException && err.name === "AbortError";
}

function clearMissTimer(): void {
  if (missTimer !== null) {
    clearTimeout(missTimer);
    missTimer = null;
  }
}

function clearPrefetchTimer(): void {
  if (prefetchTimer !== null) {
    clearTimeout(prefetchTimer);
    prefetchTimer = null;
  }
}

function preparePath(filePath: string): void {
  if (cachedPath.value !== null && cachedPath.value !== filePath) {
    clearInspectorImageCache();
  }
  cachedPath.value = filePath;
}

function scheduleIdlePrefetch(filePath: string, sel: ImageSelection): void {
  clearPrefetchTimer();
  prefetchTimer = setTimeout(() => {
    prefetchTimer = null;
    const dims = subresourceDims.value;
    if (dims === null) {
      return;
    }
    prefetchInspectorNeighbors(filePath, sel, dims);
    warmInspectorMipChain(filePath, sel, dims.levelCount);
  }, PREFETCH_IDLE_MS);
}

async function fetchMiss(
  filePath: string,
  sel: ImageSelection,
  gen: number,
): Promise<void> {
  loading.value = objectUrl.value === null;
  try {
    const url = await loadInspectorImageUrl(filePath, sel);
    if (gen !== fetchGen.value) {
      return;
    }
    objectUrl.value = url;
    scheduleIdlePrefetch(filePath, sel);
  } catch (err: unknown) {
    if (gen !== fetchGen.value || isAbortError(err)) {
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

function clearViewerState(): void {
  if (cachedPath.value !== null) {
    clearInspectorImageCache();
    cachedPath.value = null;
  }
  objectUrl.value = null;
  loading.value = false;
}

function showCachedHit(filePath: string, sel: ImageSelection, hit: string): void {
  objectUrl.value = hit;
  loading.value = false;
  scheduleIdlePrefetch(filePath, sel);
}

function scheduleMissFetch(
  filePath: string,
  sel: ImageSelection,
  gen: number,
): void {
  // Keep showing the previous layer while scrubbing; only extract after settle.
  // First paint (nothing on screen yet) skips the debounce.
  if (objectUrl.value === null) {
    void fetchMiss(filePath, sel, gen);
    return;
  }
  loading.value = false;
  missTimer = setTimeout(() => {
    missTimer = null;
    void fetchMiss(filePath, sel, gen);
  }, MISS_DEBOUNCE_MS);
}

function onSelectionChange(): void {
  const filePath = path.value;
  const gen = ++fetchGen.value;
  const sel = { ...selection.value };

  loadFailed.value = false;
  clearMissTimer();
  clearPrefetchTimer();

  if (filePath === null || filePath === "" || report.value === null) {
    clearViewerState();
    return;
  }

  preparePath(filePath);

  if (previewUnavailable.value !== null) {
    objectUrl.value = null;
    loading.value = false;
    return;
  }

  const hit = peekInspectorImageUrl(filePath, sel);
  if (hit !== null) {
    showCachedHit(filePath, sel, hit);
    return;
  }

  scheduleMissFetch(filePath, sel, gen);
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
    onSelectionChange();
  },
  { immediate: true },
);

onUnmounted(() => {
  fetchGen.value += 1;
  clearMissTimer();
  clearPrefetchTimer();
  clearInspectorImageCache();
  cachedPath.value = null;
  objectUrl.value = null;
});
</script>

<template>
  <ImageViewport
    :src="objectUrl"
    alt="KTX subresource preview"
    :loading="loading"
    :content-width="contentWidth"
    :content-height="contentHeight"
    empty-label="No image selected"
    :error-label="errorLabel"
    :unavailable-label="unavailableLabel"
    :hint="tonemapHint"
  />
</template>
