<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch, watchEffect } from "vue";
import { call } from "../../api/commands";
import { subscribe } from "../../api/events";
import { toAppError } from "../../api/guards";
import type { ExportFormat } from "../../bindings/ExportFormat";
import type { ExportSelection } from "../../bindings/ExportSelection";
import type { ImageSelection } from "../../bindings/ImageSelection";
import type { PreviewKind } from "../../bindings/PreviewKind";
import { Download } from "@lucide/vue";
import { Input } from "@/components/ui/input";
import Button from "../../shared/components/Button.vue";
import EnumSelect from "../../shared/components/EnumSelect.vue";
import type { EnumSelectOption } from "../../shared/enumSelect";
import { alertAppError } from "../../shared/alertAppError";
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

function isFormatKind(value: string): value is FormatKind {
  return value === "png" || value === "jpeg" || value === "exr";
}

function isScopeKind(value: string): value is ScopeKind {
  return value === "one" || value === "allFrames" || value === "all";
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

const scopeOptions = SCOPE_OPTIONS satisfies readonly EnumSelectOption[];

const formatOptions = computed((): EnumSelectOption[] => {
  const reason = pngJpegDisabledReason.value;
  return FORMAT_OPTIONS.filter(
    (option) => !pngJpegDisabled.value || option.value === "exr",
  ).map((option) =>
    option.value === "exr" && reason !== null
      ? { ...option, help: reason }
      : { ...option },
  );
});

const scopeModel = computed({
  get: (): string | null => scope.value,
  set: (value: string | null): void => {
    if (value !== null && isScopeKind(value)) {
      scope.value = value;
    }
  },
});

const formatModel = computed({
  get: (): string | null => formatKind.value,
  set: (value: string | null): void => {
    if (value !== null && isFormatKind(value)) {
      formatKind.value = value;
    }
  },
});

const fileReady = computed(
  (): boolean => Boolean(path.value) && report.value !== null,
);

const formatAllowed = computed(
  (): boolean => !pngJpegDisabled.value || formatKind.value === "exr",
);

const canExport = computed(
  (): boolean => fileReady.value && formatAllowed.value && !busy.value,
);

function onJpegQualityUpdate(value: string | number): void {
  const n = typeof value === "number" ? value : Number(value);
  if (!Number.isFinite(n)) {
    return;
  }
  jpegQuality.value = Math.min(100, Math.max(1, Math.round(n)));
}

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
    await alertAppError(toAppError(err));
  } finally {
    busy.value = false;
  }
}

let stopMenu: (() => void) | undefined;

onMounted(() => {
  stopMenu = subscribe("menu://action", (action) => {
    if (action === "export") {
      void onExport();
    }
  });
});

onUnmounted(() => {
  stopMenu?.();
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
  <div class="flex w-full flex-col gap-1.5">
    <div
      class="panel flex w-full flex-wrap items-center justify-end gap-2 px-2.5 py-1.5"
      role="group"
      aria-label="Export"
    >
      <label class="flex min-w-0 items-center gap-1.5 text-xs text-ink-muted">
        <span class="shrink-0">Scope</span>
        <EnumSelect
          v-model="scopeModel"
          :options="scopeOptions"
          :empty-label="false"
          class="h-8 w-[13.5rem] max-w-[40vw]"
        />
      </label>

      <label class="flex items-center gap-1.5 text-xs text-ink-muted">
        <span class="shrink-0">Format</span>
        <EnumSelect
          v-model="formatModel"
          :options="formatOptions"
          :empty-label="false"
          class="h-8 w-[5.5rem]"
        />
      </label>

      <label
        v-if="formatKind === 'jpeg'"
        class="flex items-center gap-1.5 text-xs text-ink-muted"
      >
        <span class="shrink-0">Q</span>
        <Input
          type="number"
          min="1"
          max="100"
          step="1"
          class="h-8 w-14 px-1.5 text-center font-mono tabular-nums"
          aria-label="JPEG quality"
          :model-value="String(jpegQuality)"
          @update:model-value="onJpegQualityUpdate"
        />
      </label>

      <Button
        variant="primary"
        :disabled="!canExport"
        @click="onExport"
      >
        <Download
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        {{ busy ? "Exporting…" : "Export" }}
      </Button>
    </div>

    <ul
      v-if="writtenFiles.length > 0"
      class="m-0 max-h-20 w-full list-none overflow-auto p-0 text-right text-sm text-ink"
    >
      <li
        v-for="file in writtenFiles"
        :key="file"
        class="break-all py-0.5 font-mono text-xs text-ink-muted"
      >
        {{ file }}
      </li>
    </ul>
  </div>
</template>
