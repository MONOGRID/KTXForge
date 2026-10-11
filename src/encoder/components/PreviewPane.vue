<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { call } from "../../api/commands";
import { toAppError } from "../../api/guards";
import type { EncodeResult } from "../../bindings/EncodeResult";
import type { ImageSelection } from "../../bindings/ImageSelection";
import type { InputRole } from "../../bindings/InputRole";
import ImageViewport from "../../shared/components/ImageViewport.vue";
import RangeTickLabels from "../../shared/components/RangeTickLabels.vue";
import { toastAppError } from "../../shared/toastAppError";
import { composeCubemapCross } from "../cubemapCross";
import { setDropZone } from "../dropTarget";
import { selectedOutput, store } from "../store";

type PackAxis = "layer" | "mip" | "depth" | null;

const previewIndex = ref(0);
const encodedMip = ref(0);

const sourceUrl = ref<string | null>(null);
const encodedUrl = ref<string | null>(null);
const contentWidth = ref<number | null>(null);
const contentHeight = ref<number | null>(null);
const sourceLoading = ref(false);
const encodedLoading = ref(false);
/** Face object URLs held while composing a cubemap cross (revoked with composites). */
const faceSourceUrls = ref<Array<string | null>>([
  null,
  null,
  null,
  null,
  null,
  null,
]);
const faceEncodedUrls = ref<Array<string | null>>([
  null,
  null,
  null,
  null,
  null,
  null,
]);

const role = computed((): InputRole => store.inputRole);

const packAxis = computed((): PackAxis => {
  switch (role.value) {
    case "arrayLayers":
      return "layer";
    case "mipChain":
      return "mip";
    case "depthSlices":
      return "depth";
    default:
      return null;
  }
});

const isCubemap = computed((): boolean => role.value === "cubemapFaces");

const result = computed((): EncodeResult | null => {
  const slot = selectedOutput.value;
  if (slot === null || slot.job.status !== "done") {
    return null;
  }
  return slot.job.result;
});

const selectedJobId = computed((): string | null => {
  const slot = selectedOutput.value;
  if (
    slot === null ||
    slot.job.status !== "done" ||
    slot.job.jobId === null
  ) {
    return null;
  }
  return slot.job.jobId;
});

const inputCount = computed((): number => selectedOutput.value?.inputs.length ?? 0);

const packMax = computed((): number => Math.max(0, inputCount.value - 1));

const showPackSlider = computed(
  (): boolean => packAxis.value !== null && inputCount.value > 1,
);

/** Generated-mip scrubber: after encode when KTX has multiple levels and role is not mipChain. */
const showGeneratedMipSlider = computed((): boolean => {
  if (role.value === "mipChain") {
    return false;
  }
  const levels = result.value?.levels ?? 0;
  return selectedJobId.value !== null && levels > 1;
});

const generatedMipMax = computed((): number =>
  Math.max(0, (result.value?.levels ?? 1) - 1),
);

const showTonemapHint = computed((): string | null =>
  result.value?.preview.kind === "tonemappedFromExr"
    ? "Preview, not stored data"
    : null,
);

const comparing = computed(
  (): boolean => sourceUrl.value !== null && encodedUrl.value !== null,
);

const viewportSrc = computed((): string | null => {
  if (comparing.value) {
    return sourceUrl.value;
  }
  return sourceUrl.value ?? encodedUrl.value;
});

const compareSrc = computed((): string | null =>
  comparing.value ? encodedUrl.value : null,
);

const loading = computed(
  (): boolean =>
    viewportSrc.value === null &&
    (sourceLoading.value || encodedLoading.value),
);

const packSliderLabel = computed((): string => {
  const i = previewIndex.value;
  switch (packAxis.value) {
    case "layer":
      return `Layer ${String(i)}`;
    case "mip":
      return `Mip L${String(i)}`;
    case "depth":
      return `Depth ${String(i)}`;
    default:
      return "";
  }
});

const generatedMipLabel = computed((): string => {
  const level = encodedMip.value;
  const baseW = result.value?.width ?? 0;
  const baseH = result.value?.height ?? 0;
  const w = Math.max(1, baseW >> level);
  const h = Math.max(1, baseH >> level);
  return `Mip L${String(level)} · ${String(w)}×${String(h)}`;
});

function revoke(url: string | null): void {
  if (url !== null) {
    URL.revokeObjectURL(url);
  }
}

function revokeFaceList(urls: Array<string | null>): void {
  for (const url of urls) {
    revoke(url);
  }
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${String(bytes)} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function formatMetric(value: number | null): string {
  if (value === null) {
    return "—";
  }
  return value.toFixed(4);
}

function formatMipTick(level: number): string {
  return `L${String(level)}`;
}

function clampIndex(index: number, max: number): number {
  if (max < 0) {
    return 0;
  }
  return Math.min(Math.max(0, index), max);
}

function bytesToUrl(bytes: ArrayBuffer): string {
  return URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
}

function selectedOutputId(): string | null {
  const slot = selectedOutput.value;
  return slot === null ? null : slot.id;
}

async function loadSourcePreview(path: string): Promise<string> {
  const bytes = await call("source_preview", { path });
  return bytesToUrl(bytes);
}

function currentEncodedLevel(): number {
  if (role.value === "mipChain") {
    return previewIndex.value;
  }
  if (showGeneratedMipSlider.value) {
    return encodedMip.value;
  }
  return 0;
}

function selectionForEncode(face = 0): ImageSelection {
  const level = currentEncodedLevel();
  const layer = role.value === "arrayLayers" ? previewIndex.value : 0;
  const depthSlice = role.value === "depthSlices" ? previewIndex.value : 0;
  return { level, face, layer, depthSlice };
}

function onPackInput(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  previewIndex.value = Number(el.value);
}

function onGeneratedMipInput(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  encodedMip.value = Number(el.value);
}

// Clamp indices when role / inputs / result change.
watch(
  [role, inputCount, () => result.value?.levels ?? 1],
  () => {
    previewIndex.value = clampIndex(previewIndex.value, packMax.value);
    encodedMip.value = clampIndex(encodedMip.value, generatedMipMax.value);
  },
  { immediate: true },
);

// Reset selection when switching outputs.
watch(
  () => selectedOutput.value?.id ?? null,
  () => {
    previewIndex.value = 0;
    encodedMip.value = 0;
  },
);

watch(
  [
    () => selectedOutput.value?.id ?? null,
    role,
    previewIndex,
    () =>
      selectedOutput.value?.inputs.map((s) => s.path).join("\0") ?? "",
  ],
  () => {
    const slot = selectedOutput.value;
    const loadId = slot?.id ?? null;
    const currentRole = role.value;

    revoke(sourceUrl.value);
    sourceUrl.value = null;
    revokeFaceList(faceSourceUrls.value);
    faceSourceUrls.value = [null, null, null, null, null, null];
    contentWidth.value = null;
    contentHeight.value = null;

    if (slot === null || slot.inputs.length === 0) {
      sourceLoading.value = false;
      return;
    }

    sourceLoading.value = true;
    const generation = loadId;

    void (async () => {
      try {
        if (currentRole === "cubemapFaces") {
          const urls: Array<string | null> = await Promise.all(
            Array.from({ length: 6 }, async (_, face) => {
              if (face >= slot.inputs.length) {
                return null;
              }
              try {
                return await loadSourcePreview(slot.inputs[face].path);
              } catch {
                return null;
              }
            }),
          );
          if (selectedOutputId() !== generation) {
            revokeFaceList(urls);
            return;
          }
          faceSourceUrls.value = urls;
          const composed = await composeCubemapCross(urls);
          if (selectedOutputId() !== generation) {
            revoke(composed.url);
            return;
          }
          sourceUrl.value = composed.url;
          contentWidth.value = composed.width;
          contentHeight.value = composed.height;
          return;
        }

        const index =
          packAxis.value === null ? 0 : clampIndex(previewIndex.value, packMax.value);
        if (index >= slot.inputs.length) {
          return;
        }
        const url = await loadSourcePreview(slot.inputs[index].path);
        if (selectedOutputId() !== generation) {
          revoke(url);
          return;
        }
        sourceUrl.value = url;
      } catch {
        // Leave the source slot empty.
      } finally {
        if (selectedOutputId() === generation) {
          sourceLoading.value = false;
        }
      }
    })();
  },
  { immediate: true },
);

watch(
  [
    selectedJobId,
    role,
    previewIndex,
    encodedMip,
    () => result.value?.levels ?? 0,
  ],
  () => {
    const jobId = selectedJobId.value;
    const currentRole = role.value;

    revoke(encodedUrl.value);
    encodedUrl.value = null;
    revokeFaceList(faceEncodedUrls.value);
    faceEncodedUrls.value = [null, null, null, null, null, null];

    if (jobId === null) {
      encodedLoading.value = false;
      return;
    }

    encodedLoading.value = true;
    const generation = jobId;

    void (async () => {
      try {
        if (currentRole === "cubemapFaces") {
          const urls: Array<string | null> = await Promise.all(
            Array.from({ length: 6 }, async (_, face) => {
              try {
                const bytes = await call("encoded_preview", {
                  jobId,
                  selection: selectionForEncode(face),
                });
                return bytesToUrl(bytes);
              } catch {
                return null;
              }
            }),
          );
          if (selectedJobId.value !== generation) {
            revokeFaceList(urls);
            return;
          }
          faceEncodedUrls.value = urls;
          const composed = await composeCubemapCross(urls);
          if (selectedJobId.value !== generation) {
            revoke(composed.url);
            return;
          }
          encodedUrl.value = composed.url;
          contentWidth.value = composed.width;
          contentHeight.value = composed.height;
          return;
        }

        const bytes = await call("encoded_preview", {
          jobId,
          selection: selectionForEncode(0),
        });
        if (selectedJobId.value !== generation) {
          return;
        }
        encodedUrl.value = bytesToUrl(bytes);

        const res = result.value;
        if (res !== null) {
          const level = currentEncodedLevel();
          contentWidth.value = Math.max(1, res.width >> level);
          contentHeight.value = Math.max(1, res.height >> level);
        }
      } catch (err: unknown) {
        if (selectedJobId.value !== generation) {
          return;
        }
        toastAppError(toAppError(err));
      } finally {
        if (selectedJobId.value === generation) {
          encodedLoading.value = false;
        }
      }
    })();
  },
  { immediate: true },
);

onUnmounted(() => {
  revoke(sourceUrl.value);
  revoke(encodedUrl.value);
  revokeFaceList(faceSourceUrls.value);
  revokeFaceList(faceEncodedUrls.value);
  sourceUrl.value = null;
  encodedUrl.value = null;
});
</script>

<template>
  <section
    class="panel flex h-full min-h-0 flex-col overflow-hidden"
    data-drop-zone="center"
    @pointerenter="setDropZone('center')"
  >
    <header class="shrink-0 border-b border-line px-3 py-2">
      <h2 class="m-0 text-sm font-semibold">
        Preview
      </h2>
    </header>

    <div class="flex min-h-0 flex-1 flex-col gap-2 p-3">
      <div
        v-if="showPackSlider || showGeneratedMipSlider"
        class="flex shrink-0 flex-col gap-2"
      >
        <div
          v-if="showPackSlider"
          class="flex flex-col gap-1"
        >
          <label
            class="text-sm text-ink-muted"
            for="preview-pack"
          >{{ packSliderLabel }}</label>
          <input
            id="preview-pack"
            type="range"
            class="w-full accent-brand"
            min="0"
            :max="packMax"
            step="1"
            :value="previewIndex"
            @input="onPackInput"
          >
          <RangeTickLabels
            :min="0"
            :max="packMax"
            :format="packAxis === 'mip' ? formatMipTick : undefined"
          />
        </div>

        <div
          v-if="showGeneratedMipSlider"
          class="flex flex-col gap-1"
        >
          <label
            class="text-sm text-ink-muted"
            for="preview-gen-mip"
          >{{ generatedMipLabel }}</label>
          <input
            id="preview-gen-mip"
            type="range"
            class="w-full accent-brand"
            min="0"
            :max="generatedMipMax"
            step="1"
            :value="encodedMip"
            @input="onGeneratedMipInput"
          >
          <RangeTickLabels
            :min="0"
            :max="generatedMipMax"
            :format="formatMipTick"
          />
        </div>
      </div>

      <ImageViewport
        class="min-h-0 flex-1"
        :src="viewportSrc"
        :compare-src="compareSrc"
        :content-width="contentWidth"
        :content-height="contentHeight"
        :alt="isCubemap ? 'Cubemap source' : 'Source'"
        :compare-alt="isCubemap ? 'Cubemap encoded' : 'Encoded preview'"
        :loading="loading"
        empty-label="Drop images here to add KTX outputs, or select one to preview"
        :hint="showTonemapHint"
        original-label="Original"
        result-label="Encoded"
      />

      <ScrollArea class="max-h-40 shrink-0">
        <div class="flex flex-col gap-2 pr-2">
          <dl class="m-0 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
            <dt class="text-ink-muted">
              Size
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? formatBytes(result.outputBytes) : "—" }}
            </dd>
            <dt class="text-ink-muted">
              Dimensions
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? `${String(result.width)}×${String(result.height)}` : "—" }}
            </dd>
            <dt class="text-ink-muted">
              Codec
            </dt>
            <dd class="m-0 font-mono">
              {{ result?.codec ?? "—" }}
            </dd>
            <dt class="text-ink-muted">
              Levels
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? String(result.levels) : "—" }}
            </dd>
            <dt class="text-ink-muted">
              Layers
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? String(result.layers) : "—" }}
            </dd>
            <dt class="text-ink-muted">
              Faces
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? String(result.faces) : "—" }}
            </dd>
            <dt class="text-ink-muted">
              SSIM
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? formatMetric(result.ssim) : "—" }}
            </dd>
            <dt class="text-ink-muted">
              PSNR
            </dt>
            <dd class="m-0 font-mono">
              {{ result ? formatMetric(result.psnr) : "—" }}
            </dd>
          </dl>

          <ul
            v-if="result && result.warnings.length > 0"
            class="m-0 list-disc pl-4 text-xs text-ink-muted"
          >
            <li
              v-for="(warning, index) in result.warnings"
              :key="index"
            >
              {{ warning }}
            </li>
          </ul>
        </div>
      </ScrollArea>
    </div>
  </section>
</template>
