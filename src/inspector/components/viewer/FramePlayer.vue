<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.5
import { computed, ref, watch } from "vue";
import { useIntervalFn } from "@vueuse/core";
import { call } from "../../../api/commands";
import type { KtxAnimDataInfo } from "../../../bindings/KtxAnimDataInfo";
import { store } from "../../store";

const playsCompleted = ref(0);

const anim = computed((): KtxAnimDataInfo | null => {
  const report = store.report;
  if (report === null || report.kind.kind !== "ktx2") {
    return null;
  }
  return report.kind.info.keyValueData.KTXanimData;
});

const frameCount = computed((): number => {
  const report = store.report;
  if (report === null || report.kind.kind !== "ktx2") {
    return 0;
  }
  return Math.max(1, report.kind.info.header.layerCount);
});

const frameLengthSec = computed((): number => {
  const data = anim.value;
  if (data === null || data.timescale === 0) {
    return 0;
  }
  return data.duration / data.timescale;
});

const sequenceLengthSec = computed(
  (): number => frameLengthSec.value * frameCount.value,
);

const intervalMs = computed((): number => {
  const sec = frameLengthSec.value;
  if (sec <= 0) {
    return 1000;
  }
  return Math.max(1, Math.round(sec * 1000));
});

const visible = computed((): boolean => anim.value !== null);

function formatSeconds(value: number): string {
  if (!Number.isFinite(value)) {
    return "—";
  }
  return `${value.toFixed(3)} s`;
}

function stepBy(delta: number): void {
  const count = frameCount.value;
  if (count <= 0) {
    return;
  }
  const next = store.selection.layer + delta;
  if (next < 0 || next >= count) {
    return;
  }
  store.selection.layer = next;
}

function advanceFrame(): void {
  const data = anim.value;
  const count = frameCount.value;
  if (data === null || count <= 0) {
    pause();
    return;
  }

  if (store.selection.layer < count - 1) {
    store.selection.layer += 1;
    return;
  }

  // Finished one play-through of the sequence.
  playsCompleted.value += 1;
  if (data.loopCount === 0) {
    store.selection.layer = 0;
    return;
  }
  if (playsCompleted.value >= data.loopCount) {
    pause();
    return;
  }
  store.selection.layer = 0;
}

const { pause, resume, isActive } = useIntervalFn(
  () => {
    advanceFrame();
  },
  intervalMs,
  { immediate: false },
);

function togglePlay(): void {
  if (isActive.value) {
    pause();
    return;
  }
  playsCompleted.value = 0;
  resume();
}

async function preloadNext(): Promise<void> {
  const path = store.path;
  const data = anim.value;
  if (path === null || path === "" || data === null) {
    return;
  }
  const count = frameCount.value;
  if (count <= 1) {
    return;
  }
  const nextLayer = (store.selection.layer + 1) % count;
  try {
    await call("inspector_image", {
      path,
      selection: {
        level: store.selection.level,
        face: store.selection.face,
        layer: nextLayer,
        depthSlice: store.selection.depthSlice,
      },
    });
  } catch {
    // Preload is best-effort; ImageViewer surfaces real load errors.
  }
}

watch(
  () =>
    [
      store.path,
      store.selection.level,
      store.selection.face,
      store.selection.layer,
      store.selection.depthSlice,
      anim.value,
    ] as const,
  () => {
    void preloadNext();
  },
);

watch(visible, (isVisible) => {
  if (!isVisible) {
    pause();
    playsCompleted.value = 0;
  }
});
</script>

<template>
  <div
    v-if="visible && anim !== null"
    class="flex flex-col gap-2"
  >
    <div class="text-sm text-ink-muted">
      Animation
    </div>
    <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs font-mono text-ink-faint">
      <span>Frames {{ frameCount }}</span>
      <span>Frame {{ formatSeconds(frameLengthSec) }}</span>
      <span>Sequence {{ formatSeconds(sequenceLengthSec) }}</span>
      <span>Loop {{ anim.loopCount === 0 ? "∞" : anim.loopCount }}</span>
    </div>
    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        class="btn"
        :disabled="store.selection.layer <= 0"
        @click="stepBy(-1)"
      >
        Prev
      </button>
      <button
        type="button"
        class="btn btn-primary"
        @click="togglePlay"
      >
        {{ isActive ? "Pause" : "Play" }}
      </button>
      <button
        type="button"
        class="btn"
        :disabled="store.selection.layer >= frameCount - 1 && anim.loopCount !== 0"
        @click="stepBy(1)"
      >
        Next
      </button>
      <span class="text-sm font-mono text-ink-muted">
        {{ store.selection.layer + 1 }} / {{ frameCount }}
      </span>
    </div>
  </div>
</template>
