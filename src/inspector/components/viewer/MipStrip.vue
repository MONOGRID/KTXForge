<script setup lang="ts">
import { computed } from "vue";
import { report, selection } from "../../store";
import RangeTickLabels from "../../../shared/components/RangeTickLabels.vue";

function formatMipTick(level: number): string {
  return `L${String(level)}`;
}

type MipEntry = {
  level: number;
  width: number;
  height: number;
};

function mipSize(base: number, level: number): number {
  return Math.max(1, base >> level);
}

const entries = computed((): MipEntry[] => {
  const current = report.value;
  if (current === null) {
    return [];
  }

  const baseW =
    current.kind.kind === "ktx2"
      ? current.kind.info.header.pixelWidth
      : current.kind.header.pixelWidth;
  const baseH =
    current.kind.kind === "ktx2"
      ? current.kind.info.header.pixelHeight
      : current.kind.header.pixelHeight;
  const levelCount =
    current.kind.kind === "ktx2"
      ? Math.max(1, current.kind.info.header.levelCount)
      : Math.max(1, current.kind.header.numberOfMipmapLevels);

  const list: MipEntry[] = [];
  for (let i = 0; i < levelCount; i += 1) {
    list.push({
      level: i,
      width: mipSize(baseW, i),
      height: mipSize(baseH, i),
    });
  }
  return list;
});

const maxLevel = computed((): number => Math.max(0, entries.value.length - 1));

const currentEntry = computed((): MipEntry | null => {
  const list = entries.value;
  if (list.length === 0) {
    return null;
  }
  const level = Math.min(Math.max(0, selection.value.level), list.length - 1);
  return list.find((entry) => entry.level === level) ?? null;
});

const label = computed((): string => {
  const entry = currentEntry.value;
  if (entry === null) {
    return "Mip";
  }
  return `Mip L${String(entry.level)} · ${String(entry.width)}×${String(entry.height)}`;
});

function onLevelInput(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  selection.value.level = Number(el.value);
}
</script>

<template>
  <div
    v-if="entries.length > 0"
    class="flex flex-col gap-1"
  >
    <label
      class="text-sm text-ink-muted"
      for="inspector-mip"
    >{{ label }}</label>
    <input
      id="inspector-mip"
      type="range"
      class="w-full accent-brand"
      min="0"
      :max="maxLevel"
      step="1"
      :value="selection.level"
      :disabled="maxLevel === 0"
      @input="onLevelInput"
    >
    <RangeTickLabels
      :min="0"
      :max="maxLevel"
      :format="formatMipTick"
    />
  </div>
</template>
