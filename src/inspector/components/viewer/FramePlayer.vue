<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useIntervalFn } from "@vueuse/core";
import { call } from "../../../api/commands";
import type { KtxAnimDataInfo } from "../../../bindings/KtxAnimDataInfo";
import { path, report, selection } from "../../store";

const playsCompleted = ref(0);

const anim = computed((): KtxAnimDataInfo | null => {
  const current = report.value;
  if (current === null || current.kind.kind !== "ktx2") {
    return null;
  }
  return current.kind.info.keyValueData.KTXanimData;
});

const frameCount = computed((): number => {
  const current = report.value;
  if (current === null || current.kind.kind !== "ktx2") {
    return 0;
  }
  return Math.max(1, current.kind.info.header.layerCount);
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
  const next = selection.value.layer + delta;
  if (next < 0 || next >= count) {
    return;
  }
  selection.value.layer = next;
}

function advanceFrame(): void {
  const data = anim.value;
  const count = frameCount.value;
  if (data === null || count <= 0) {
    pause();
    return;
  }

  if (selection.value.layer < count - 1) {
    selection.value.layer += 1;
    return;
  }

  // Finished one play-through of the sequence.
  playsCompleted.value += 1;
  if (data.loopCount === 0) {
    selection.value.layer = 0;
    return;
  }
  if (playsCompleted.value >= data.loopCount) {
    pause();
    return;
  }
  selection.value.layer = 0;
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
  const filePath = path.value;
  const data = anim.value;
  if (filePath === null || filePath === "" || data === null) {
    return;
  }
  const count = frameCount.value;
  if (count <= 1) {
    return;
  }
  const nextLayer = (selection.value.layer + 1) % count;
  try {
    await call("inspector_image", {
      path: filePath,
      selection: {
        level: selection.value.level,
        face: selection.value.face,
        layer: nextLayer,
        depthSlice: selection.value.depthSlice,
      },
    });
  } catch {
    // Preload is best-effort; ImageViewer surfaces real load errors.
  }
}

watch(
  () =>
    [
      path.value,
      selection.value.level,
      selection.value.face,
      selection.value.layer,
      selection.value.depthSlice,
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
        :disabled="selection.layer <= 0"
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
        :disabled="selection.layer >= frameCount - 1 && anim.loopCount !== 0"
        @click="stepBy(1)"
      >
        Next
      </button>
      <span class="text-sm font-mono text-ink-muted">
        {{ selection.layer + 1 }} / {{ frameCount }}
      </span>
    </div>
  </div>
</template>
