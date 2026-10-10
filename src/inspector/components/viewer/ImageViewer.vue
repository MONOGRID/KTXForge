<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.5
import { computed, onUnmounted, ref, watch } from "vue";
import { call } from "../../../api/commands";
import { toAppError } from "../../../api/guards";
import type { AppError } from "../../../bindings/AppError";
import { store } from "../../store";

type FitMode = "real" | "fit";

const fitMode = ref<FitMode>("fit");
const objectUrl = ref<string | null>(null);
const naturalWidth = ref(0);
const naturalHeight = ref(0);
const loading = ref(false);
const error = ref<AppError | null>(null);
const fetchGen = ref(0);

const previewUnavailable = computed((): string | null => {
  const preview = store.report?.preview;
  if (preview?.kind === "unavailable") {
    return preview.reason;
  }
  return null;
});

const tonemapped = computed(
  (): boolean => store.report?.preview.kind === "tonemappedFromExr",
);

function revokeUrl(): void {
  if (objectUrl.value !== null) {
    URL.revokeObjectURL(objectUrl.value);
    objectUrl.value = null;
  }
}

async function loadImage(): Promise<void> {
  const path = store.path;
  const gen = ++fetchGen.value;

  revokeUrl();
  naturalWidth.value = 0;
  naturalHeight.value = 0;
  error.value = null;

  if (path === null || path === "" || store.report === null) {
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
      path,
      selection: {
        level: store.selection.level,
        face: store.selection.face,
        layer: store.selection.layer,
        depthSlice: store.selection.depthSlice,
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
    error.value = toAppError(err);
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

function setFit(mode: FitMode): void {
  fitMode.value = mode;
}

watch(
  () =>
    [
      store.path,
      store.report,
      store.selection.level,
      store.selection.face,
      store.selection.layer,
      store.selection.depthSlice,
    ] as const,
  () => {
    void loadImage();
  },
  { immediate: true },
);

onUnmounted(() => {
  fetchGen.value += 1;
  revokeUrl();
});
</script>

<template>
  <div class="flex flex-col gap-2">
    <div class="flex flex-wrap items-center gap-2">
      <span class="text-sm text-ink-muted">View</span>
      <button
        type="button"
        class="btn"
        :class="{ 'btn-primary': fitMode === 'real' }"
        :aria-pressed="fitMode === 'real'"
        @click="setFit('real')"
      >
        1:1
      </button>
      <button
        type="button"
        class="btn"
        :class="{ 'btn-primary': fitMode === 'fit' }"
        :aria-pressed="fitMode === 'fit'"
        @click="setFit('fit')"
      >
        Fit
      </button>
      <span
        v-if="naturalWidth > 0"
        class="text-xs text-ink-faint font-mono"
      >
        {{ naturalWidth }}×{{ naturalHeight }}
      </span>
    </div>

    <p
      v-if="tonemapped"
      class="hint m-0"
    >
      Preview, not stored data
    </p>

    <div
      class="viewer-frame relative min-h-48 flex items-center justify-center overflow-auto rounded border border-line bg-surface-sunken"
    >
      <div
        v-if="loading"
        class="text-sm text-ink-muted"
      >
        Loading…
      </div>
      <div
        v-else-if="previewUnavailable !== null"
        class="p-4 text-sm text-ink-muted"
      >
        Preview unavailable: {{ previewUnavailable }}
      </div>
      <div
        v-else-if="error !== null"
        class="p-4 text-sm text-danger"
      >
        {{ error.message }}
      </div>
      <div
        v-else-if="objectUrl === null"
        class="p-4 text-sm text-ink-faint"
      >
        No image selected
      </div>
      <div
        v-else
        class="checkerboard m-2 inline-block max-w-full"
      >
        <img
          :src="objectUrl"
          alt="KTX subresource preview"
          class="viewer-img block"
          :class="fitMode === 'fit' ? 'viewer-img-fit' : 'viewer-img-real'"
          @load="onImgLoad"
        >
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
}

.viewer-img-real {
  width: auto;
  height: auto;
  max-width: none;
}

.viewer-img-fit {
  max-width: min(100%, 100%);
  max-height: min(60vh, 480px);
  width: auto;
  height: auto;
  object-fit: contain;
}
</style>
