<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, watchEffect } from "vue";
import { call } from "../../api/commands";
import { on } from "../../api/events";
import { toAppError } from "../../api/guards";
import type { ExportFormat } from "../../bindings/ExportFormat";
import type { ExportSelection } from "../../bindings/ExportSelection";
import type { ImageSelection } from "../../bindings/ImageSelection";
import type { PreviewKind } from "../../bindings/PreviewKind";
import Button from "../../shared/components/Button.vue";
import Panel from "../../shared/components/Panel.vue";
import { toastAppError } from "../../shared/toastAppError";
import { path, report, selection } from "../store";

type ScopeKind = "one" | "allFrames" | "all";
type FormatKind = "png" | "jpeg" | "exr";

const SCOPE_OPTIONS = [
  { value: "one", label: "Current image" },
  { value: "allFrames", label: "All frames at current level" },
  { value: "all", label: "Everything" },
] as const satisfies ReadonlyArray<{ value: ScopeKind; label: string }>;

const FORMAT_OPTIONS = [
  { value: "png", label: "PNG" },
  { value: "jpeg", label: "JPEG" },
  { value: "exr", label: "EXR" },
] as const satisfies ReadonlyArray<{ value: FormatKind; label: string }>;

const HDR_LDR_REASON =
  "HDR data is extracted as EXR; PNG and JPEG are not available";

function ldrExportBlockReason(
  preview: PreviewKind | undefined,
): string | null {
  if (preview === undefined || preview.kind === "png") {
    return null;
  }
  if (preview.kind === "unavailable") {
    return preview.reason;
  }
  return HDR_LDR_REASON;
}

function isLdrFormat(kind: FormatKind): boolean {
  return kind === "png" || kind === "jpeg";
}

function toExportFormat(kind: FormatKind, quality: number): ExportFormat {
  if (kind === "jpeg") {
    return { kind: "jpeg", quality };
  }
  if (kind === "exr") {
    return { kind: "exr" };
  }
  return { kind: "png" };
}

function toExportSelection(
  scopeKind: ScopeKind,
  image: ImageSelection,
): ExportSelection {
  if (scopeKind === "all") {
    return { kind: "all" };
  }
  if (scopeKind === "allFrames") {
    return { kind: "allFrames", level: image.level };
  }
  return { kind: "one", selection: { ...image } };
}

const scope = ref<ScopeKind>("one");
const formatKind = ref<FormatKind>("png");
const jpegQuality = ref(90);
const busy = ref(false);
const writtenFiles = ref<string[]>([]);

const pngJpegDisabledReason = computed((): string | null =>
  ldrExportBlockReason(report.value?.preview),
);

const pngJpegDisabled = computed(
  (): boolean => pngJpegDisabledReason.value !== null,
);

watch(
  pngJpegDisabled,
  (disabled) => {
    if (disabled) {
      formatKind.value = "exr";
    }
  },
  { immediate: true },
);

const fileReady = computed(
  (): boolean => Boolean(path.value) && report.value !== null,
);

const formatAllowed = computed(
  (): boolean => !pngJpegDisabled.value || formatKind.value === "exr",
);

const canExport = computed(
  (): boolean => fileReady.value && formatAllowed.value && !busy.value,
);

async function onExport(): Promise<void> {
  if (!canExport.value) {
    return;
  }

  writtenFiles.value = [];

  const filePath = path.value;
  if (filePath === null || !fileReady.value) {
    return;
  }

  busy.value = true;
  try {
    const result = await call("export_images", {
      request: {
        path: filePath,
        selection: toExportSelection(scope.value, selection.value),
        format: toExportFormat(formatKind.value, jpegQuality.value),
      },
    });
    writtenFiles.value = result.files;
  } catch (err: unknown) {
    toastAppError(toAppError(err));
  } finally {
    busy.value = false;
  }
}

const unlisteners: Array<() => void> = [];

onMounted(() => {
  void on("menu://action", (action) => {
    if (action === "export") {
      void onExport();
    }
  }).then((unlisten) => {
    unlisteners.push(unlisten);
  });
});

onUnmounted(() => {
  for (const unlisten of unlisteners) {
    unlisten();
  }
});

watchEffect(() => {
  void call("set_menu_item_enabled", {
    id: "export",
    enabled: canExport.value,
  }).catch(() => {
    // Menu may be unavailable on some platforms / early boot.
  });
});
</script>

<template>
  <Panel
    title="Export"
    :collapsible="false"
  >
    <div class="field-row">
      <div class="label">
        Scope
      </div>
      <div class="min-w-0 flex-1 flex flex-col gap-1">
        <label
          v-for="option in SCOPE_OPTIONS"
          :key="option.value"
          class="flex items-center gap-2 text-sm text-ink"
        >
          <input
            v-model="scope"
            type="radio"
            name="export-scope"
            :value="option.value"
          >
          {{ option.label }}
        </label>
      </div>
    </div>

    <div class="field-row">
      <div class="label">
        Format
      </div>
      <div class="min-w-0 flex-1 flex flex-col gap-1">
        <label
          v-for="option in FORMAT_OPTIONS"
          :key="option.value"
          class="flex items-center gap-2 text-sm text-ink"
          :class="{ 'opacity-50': pngJpegDisabled && isLdrFormat(option.value) }"
        >
          <input
            v-model="formatKind"
            type="radio"
            name="export-format"
            :value="option.value"
            :disabled="pngJpegDisabled && isLdrFormat(option.value)"
          >
          {{ option.label }}
        </label>
        <p
          v-if="pngJpegDisabledReason"
          class="hint"
        >
          {{ pngJpegDisabledReason }}
        </p>
      </div>
    </div>

    <div
      v-if="formatKind === 'jpeg'"
      class="field-row"
    >
      <div class="label">
        JPEG quality
      </div>
      <div class="min-w-0 flex-1">
        <div class="flex items-center gap-2">
          <input
            v-model.number="jpegQuality"
            type="range"
            min="1"
            max="100"
            step="1"
            class="min-w-0 flex-1"
          >
          <span class="w-8 shrink-0 text-right text-sm text-ink-muted tabular-nums">
            {{ jpegQuality }}
          </span>
        </div>
      </div>
    </div>

    <div class="mt-2 flex items-center gap-2">
      <Button
        variant="primary"
        :disabled="!canExport"
        @click="onExport"
      >
        {{ busy ? "Exporting…" : "Export" }}
      </Button>
    </div>

    <ul
      v-if="writtenFiles.length > 0"
      class="mt-3 m-0 max-h-24 list-none overflow-auto p-0 text-sm text-ink"
    >
      <li
        v-for="file in writtenFiles"
        :key="file"
        class="break-all border-t border-line py-1 font-mono text-xs first:border-t-0"
      >
        {{ file }}
      </li>
    </ul>
  </Panel>
</template>
