<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { call } from "../../api/commands";
import { toAppError } from "../../api/guards";
import type { EncodeResult } from "../../bindings/EncodeResult";
import ImageViewport from "../../shared/components/ImageViewport.vue";
import { toastAppError } from "../../shared/toastAppError";
import { store } from "../store";

const sourceUrl = ref<string | null>(null);
const encodedUrl = ref<string | null>(null);
const sourceLoading = ref(false);
const encodedLoading = ref(false);

const result = computed((): EncodeResult | null => store.job.result);

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

function sourcePathAt(index: number): string | undefined {
  if (index < 0 || index >= store.sources.length) {
    return undefined;
  }
  return store.sources[index].path;
}

function revoke(url: string | null): void {
  if (url !== null) {
    URL.revokeObjectURL(url);
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

watch(
  () => sourcePathAt(0),
  (path) => {
    revoke(sourceUrl.value);
    sourceUrl.value = null;
    if (path === undefined) {
      sourceLoading.value = false;
      return;
    }
    sourceLoading.value = true;
    void call("source_preview", { path })
      .then((bytes) => {
        if (sourcePathAt(0) !== path) {
          return;
        }
        sourceUrl.value = URL.createObjectURL(
          new Blob([bytes], { type: "image/png" }),
        );
      })
      .catch(() => {
        // Leave the source slot empty.
      })
      .finally(() => {
        if (sourcePathAt(0) === path) {
          sourceLoading.value = false;
        }
      });
  },
  { immediate: true },
);

watch(
  () =>
    store.job.status === "done" && store.job.jobId !== null
      ? store.job.jobId
      : null,
  (jobId) => {
    revoke(encodedUrl.value);
    encodedUrl.value = null;
    if (jobId === null) {
      encodedLoading.value = false;
      return;
    }
    encodedLoading.value = true;
    void call("encoded_preview", { jobId })
      .then((bytes) => {
        if (store.job.jobId !== jobId || store.job.status !== "done") {
          return;
        }
        encodedUrl.value = URL.createObjectURL(
          new Blob([bytes], { type: "image/png" }),
        );
      })
      .catch((err: unknown) => {
        if (store.job.jobId !== jobId || store.job.status !== "done") {
          return;
        }
        toastAppError(toAppError(err));
      })
      .finally(() => {
        if (store.job.jobId === jobId && store.job.status === "done") {
          encodedLoading.value = false;
        }
      });
  },
  { immediate: true },
);

onUnmounted(() => {
  revoke(sourceUrl.value);
  revoke(encodedUrl.value);
  sourceUrl.value = null;
  encodedUrl.value = null;
});
</script>

<template>
  <section class="panel flex h-full min-h-0 flex-col overflow-hidden">
    <header class="shrink-0 border-b border-line px-3 py-2">
      <h2 class="m-0 text-sm font-semibold">
        Preview
      </h2>
    </header>

    <div class="flex min-h-0 flex-1 flex-col gap-2 p-3">
      <ImageViewport
        class="min-h-0 flex-1"
        :src="viewportSrc"
        :compare-src="compareSrc"
        alt="Source"
        compare-alt="Encoded preview"
        :loading="loading"
        empty-label="Drop or add a source to preview"
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
