<script setup lang="ts">
import {
  Activity,
  Binary,
  Cuboid,
  Gauge,
  HardDrive,
  Layers,
  Pyramid,
  Scaling,
} from "@lucide/vue";
import { computed, type Component } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { EncodeResult } from "../../bindings/EncodeResult";

const props = defineProps<{
  result: EncodeResult | null;
}>();

type MetaChip = {
  key: string;
  label: string;
  value: string;
  icon: Component;
};

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${String(bytes)} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function formatMetric(value: number): string {
  return value.toFixed(4);
}

const chips = computed((): MetaChip[] => {
  const result = props.result;
  const items: MetaChip[] = [
    {
      key: "size",
      label: "Size",
      value: result ? formatBytes(result.outputBytes) : "—",
      icon: HardDrive,
    },
    {
      key: "dimensions",
      label: "Dimensions",
      value: result ? `${String(result.width)}×${String(result.height)}` : "—",
      icon: Scaling,
    },
    {
      key: "codec",
      label: "Codec",
      value: result?.codec ?? "—",
      icon: Binary,
    },
    {
      key: "levels",
      label: "Levels",
      value: result ? String(result.levels) : "—",
      icon: Pyramid,
    },
    {
      key: "layers",
      label: "Layers",
      value: result ? String(result.layers) : "—",
      icon: Layers,
    },
    {
      key: "faces",
      label: "Faces",
      value: result ? String(result.faces) : "—",
      icon: Cuboid,
    },
  ];

  if (result !== null && result.ssim !== null) {
    items.push({
      key: "ssim",
      label: "SSIM",
      value: formatMetric(result.ssim),
      icon: Gauge,
    });
  }

  if (result !== null && result.psnr !== null) {
    items.push({
      key: "psnr",
      label: "PSNR",
      value: formatMetric(result.psnr),
      icon: Activity,
    });
  }

  return items;
});
</script>

<template>
  <ScrollArea
    class="min-h-0 max-h-[300px] shrink-0 [&_[data-reka-scroll-area-viewport]]:max-h-[300px]"
  >
    <div class="flex flex-col gap-2 pr-2">
      <ul class="m-0 flex list-none flex-wrap gap-1.5 p-0">
        <li
          v-for="chip in chips"
          :key="chip.key"
          class="inline-flex items-center gap-1.5 rounded-md border border-transparent bg-surface-sunken px-2 py-1 text-xs shadow-sm"
        >
          <component
            :is="chip.icon"
            class="size-3.5 shrink-0 text-ink-muted"
            aria-hidden="true"
          />
          <span class="text-ink-muted">
            {{ chip.label }}
          </span>
          <span class="font-mono">
            {{ chip.value }}
          </span>
        </li>
      </ul>

      <ul
        v-if="result && result.warnings.length > 0"
        class="m-0 list-disc break-words pl-4 text-xs text-ink-muted"
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
</template>
