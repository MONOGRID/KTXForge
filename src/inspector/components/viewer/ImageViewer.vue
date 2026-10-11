<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { toAppError } from "../../../api/guards";
import type { ImageSelection } from "../../../bindings/ImageSelection";
import DepthVolumeView from "../../../shared/components/DepthVolumeView.vue";
import ImageViewport from "../../../shared/components/ImageViewport.vue";
import { toastAppError } from "../../../shared/toastAppError";
import {
  clearInspectorImageCache,
  loadInspectorImageUrl,
  peekInspectorImageUrl,
  prefetchInspectorNeighbors,
  releaseInspectorImage,
  retainInspectorImage,
  touchInspectorImageUrl,
  warmInspectorMipChain,
  type SubresourceDims,
} from "../../imageCache";
import { path, report, selection } from "../../store";
import { useCubemapCross } from "../../useCubemapCross";
import { useDepthVolume } from "../../useDepthVolume";

const MISS_DEBOUNCE_MS = 120;
const PREFETCH_IDLE_MS = 200;

const objectUrl = ref<string | null>(null);
const loading = ref(false);
const loadFailed = ref(false);
const fetchGen = ref(0);
const cachedPath = ref<string | null>(null);

let missTimer: ReturnType<typeof setTimeout> | null = null;
let prefetchTimer: ReturnType<typeof setTimeout> | null = null;
/** Flat viewport pin so LRU cannot revoke the displayed blob. */
let pinnedFlat: { filePath: string; sel: ImageSelection } | null = null;

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

function preparePath(filePath: string): void {
  if (cachedPath.value !== null && cachedPath.value !== filePath) {
    clearInspectorImageCache();
  }
  cachedPath.value = filePath;
}

const {
  depthSliceUrls,
  volumeLoading,
  showDepthVolume,
  activeDepthIndex,
  resetVolume,
  invalidateVolume,
} = useDepthVolume({
  path,
  report,
  selection,
  previewUnavailable,
  preparePath,
});

const {
  showCubemapCross,
  crossUrl,
  crossWidth,
  crossHeight,
  crossLoading,
  invalidateCross,
} = useCubemapCross({
  path,
  report,
  selection,
  previewUnavailable,
  showDepthVolume,
  preparePath,
});

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
const faceContentWidth = computed((): number | null => {
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

const faceContentHeight = computed((): number | null => {
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

const viewportSrc = computed((): string | null =>
  showCubemapCross.value ? crossUrl.value : objectUrl.value,
);

const viewportLoading = computed((): boolean =>
  showCubemapCross.value ? crossLoading.value : loading.value,
);

const viewportWidth = computed((): number | null =>
  showCubemapCross.value ? crossWidth.value : faceContentWidth.value,
);

const viewportHeight = computed((): number | null =>
  showCubemapCross.value ? crossHeight.value : faceContentHeight.value,
);

const viewportAlt = computed((): string =>
  showCubemapCross.value ? "Cubemap cross" : "KTX subresource preview",
);

const viewportHint = computed((): string | null => {
  if (showCubemapCross.value) {
    return tonemapHint.value ?? "Cubemap cross";
  }
  return tonemapHint.value;
});

function isAbortError(err: unknown): boolean {
  return err instanceof DOMException && err.name === "AbortError";
}

function releaseFlatPin(): void {
  if (pinnedFlat === null) {
    return;
  }
  releaseInspectorImage(pinnedFlat.filePath, pinnedFlat.sel);
  pinnedFlat = null;
}

function bindFlatPin(filePath: string, sel: ImageSelection): void {
  const same =
    pinnedFlat !== null &&
    pinnedFlat.filePath === filePath &&
    pinnedFlat.sel.level === sel.level &&
    pinnedFlat.sel.face === sel.face &&
    pinnedFlat.sel.layer === sel.layer &&
    pinnedFlat.sel.depthSlice === sel.depthSlice;
  if (same) {
    touchInspectorImageUrl(filePath, sel);
    return;
  }
  releaseFlatPin();
  retainInspectorImage(filePath, sel);
  touchInspectorImageUrl(filePath, sel);
  pinnedFlat = { filePath, sel: { ...sel } };
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
  // Protect the in-flight key without dropping the pin on the still-visible frame.
  retainInspectorImage(filePath, sel);
  try {
    const url = await loadInspectorImageUrl(filePath, sel);
    if (gen !== fetchGen.value) {
      releaseInspectorImage(filePath, sel);
      return;
    }
    releaseInspectorImage(filePath, sel);
    bindFlatPin(filePath, sel);
    objectUrl.value = url;
    scheduleIdlePrefetch(filePath, sel);
  } catch (err: unknown) {
    releaseInspectorImage(filePath, sel);
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
  releaseFlatPin();
  if (cachedPath.value !== null) {
    clearInspectorImageCache();
    cachedPath.value = null;
  }
  objectUrl.value = null;
  loading.value = false;
  resetVolume();
  invalidateCross();
}

function showCachedHit(filePath: string, sel: ImageSelection, hit: string): void {
  bindFlatPin(filePath, sel);
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

function clearFlatPreview(): void {
  releaseFlatPin();
  objectUrl.value = null;
  loading.value = false;
}

function selectionFilePath(): string | null {
  const filePath = path.value;
  if (filePath === null || filePath === "" || report.value === null) {
    return null;
  }
  return filePath;
}

function onSelectionChange(): void {
  const gen = ++fetchGen.value;
  const sel = { ...selection.value };

  loadFailed.value = false;
  clearMissTimer();
  clearPrefetchTimer();

  const filePath = selectionFilePath();
  if (filePath === null) {
    clearViewerState();
    return;
  }

  preparePath(filePath);

  if (previewUnavailable.value !== null || showCubemapCross.value) {
    // Cubemap preview is the shared cross; face buttons only affect export selection.
    clearFlatPreview();
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
      showCubemapCross.value,
    ] as const,
  () => {
    onSelectionChange();
  },
  { immediate: true },
);

onUnmounted(() => {
  fetchGen.value += 1;
  invalidateVolume();
  invalidateCross();
  clearMissTimer();
  clearPrefetchTimer();
  releaseFlatPin();
  clearInspectorImageCache();
  cachedPath.value = null;
  objectUrl.value = null;
});
</script>

<template>
  <div
    v-if="showDepthVolume"
    class="flex h-full min-h-0 flex-col gap-2"
  >
    <DepthVolumeView
      :slice-urls="depthSliceUrls"
      :active-index="activeDepthIndex"
      :loading="volumeLoading"
      empty-label="No depth slices"
    />
    <ImageViewport
      class="min-h-0 flex-1"
      :src="objectUrl"
      alt="KTX depth slice preview"
      :loading="loading"
      :content-width="faceContentWidth"
      :content-height="faceContentHeight"
      empty-label="Select a depth slice"
      :error-label="errorLabel"
      :unavailable-label="unavailableLabel"
      :hint="tonemapHint"
    />
  </div>
  <ImageViewport
    v-else
    :src="viewportSrc"
    :alt="viewportAlt"
    :loading="viewportLoading"
    :content-width="viewportWidth"
    :content-height="viewportHeight"
    empty-label="No image selected"
    :error-label="errorLabel"
    :unavailable-label="unavailableLabel"
    :hint="viewportHint"
  />
</template>
