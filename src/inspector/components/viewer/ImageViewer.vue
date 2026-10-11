<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { call } from "../../../api/commands";
import { toAppError } from "../../../api/guards";
import ImageViewport from "../../../shared/components/ImageViewport.vue";
import { toastAppError } from "../../../shared/toastAppError";
import { path, report, selection } from "../../store";

const objectUrl = ref<string | null>(null);
const loading = ref(false);
const loadFailed = ref(false);
const fetchGen = ref(0);

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

function revokeUrl(): void {
  if (objectUrl.value !== null) {
    URL.revokeObjectURL(objectUrl.value);
    objectUrl.value = null;
  }
}

async function loadImage(): Promise<void> {
  const filePath = path.value;
  const gen = ++fetchGen.value;

  revokeUrl();
  loadFailed.value = false;

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
    objectUrl.value = URL.createObjectURL(
      new Blob([bytes], { type: "image/png" }),
    );
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

onUnmounted(() => {
  fetchGen.value += 1;
  revokeUrl();
});
</script>

<template>
  <ImageViewport
    :src="objectUrl"
    alt="KTX subresource preview"
    :loading="loading"
    empty-label="No image selected"
    :error-label="errorLabel"
    :unavailable-label="unavailableLabel"
    :hint="tonemapHint"
  />
</template>
