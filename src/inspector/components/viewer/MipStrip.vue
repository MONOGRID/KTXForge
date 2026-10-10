<script setup lang="ts">
import { computed } from "vue";
import { report, selection } from "../../store";

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

function selectLevel(level: number): void {
  selection.value.level = level;
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
        :class="{ 'btn-primary': selection.level === entry.level }"
        :aria-pressed="selection.level === entry.level"
        @click="selectLevel(entry.level)"
      >
        L{{ entry.level }}
        <span class="text-ink-faint">{{ entry.width }}×{{ entry.height }}</span>
      </button>
    </div>
  </div>
</template>
