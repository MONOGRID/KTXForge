<script setup lang="ts">
import { ScrollArea } from "@/components/ui/scroll-area";
import type { EncodeResult } from "../../bindings/EncodeResult";

defineProps<{
  result: EncodeResult | null;
}>();

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
</script>

<template>
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
</template>
