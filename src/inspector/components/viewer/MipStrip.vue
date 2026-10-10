<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.5
import { computed } from "vue";
import { store } from "../../store";

type MipEntry = {
  level: number;
  width: number;
  height: number;
};

function mipSize(base: number, level: number): number {
  return Math.max(1, base >> level);
}

const entries = computed((): MipEntry[] => {
  const report = store.report;
  if (report === null) {
    return [];
  }

  const baseW =
    report.kind.kind === "ktx2"
      ? report.kind.info.header.pixelWidth
      : report.kind.header.pixelWidth;
  const baseH =
    report.kind.kind === "ktx2"
      ? report.kind.info.header.pixelHeight
      : report.kind.header.pixelHeight;
  const levelCount =
    report.kind.kind === "ktx2"
      ? Math.max(1, report.kind.info.header.levelCount)
      : Math.max(1, report.kind.header.numberOfMipmapLevels);

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

function selectLevel(level: number): void {
  store.selection.level = level;
}
</script>

<template>
  <div
    v-if="entries.length > 0"
    class="flex flex-col gap-1"
  >
    <div class="text-sm text-ink-muted">
      Mip levels
    </div>
    <div class="flex flex-wrap gap-1">
      <button
        v-for="entry in entries"
        :key="entry.level"
        type="button"
        class="btn font-mono text-xs"
        :class="{ 'btn-primary': store.selection.level === entry.level }"
        :aria-pressed="store.selection.level === entry.level"
        @click="selectLevel(entry.level)"
      >
        L{{ entry.level }}
        <span class="text-ink-faint">{{ entry.width }}×{{ entry.height }}</span>
      </button>
    </div>
  </div>
</template>
