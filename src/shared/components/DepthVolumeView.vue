<script setup lang="ts">
import { useElementSize } from "@vueuse/core";
import { computed, useTemplateRef } from "vue";
import { useOrbitDrag } from "../useOrbitDrag";

const props = withDefaults(
  defineProps<{
    sliceUrls: ReadonlyArray<string | null>;
    activeIndex: number;
    loading?: boolean;
    emptyLabel?: string;
  }>(),
  {
    loading: false,
    emptyLabel: "No depth slices",
  },
);

const sceneRef = useTemplateRef("sceneRef");
const { width: sceneW, height: sceneH } = useElementSize(sceneRef);

const {
  dragging,
  volumeTransform,
  onPointerDown,
  onPointerMove,
  onPointerUp,
  resetOrbit,
} = useOrbitDrag({
  initialRotX: 58,
  initialRotZ: -36,
});

/** Plane edge in CSS px; sized from the scene so the stack fits the panel. */
const planeSize = computed((): number => {
  const w = sceneW.value;
  const h = sceneH.value;
  if (w <= 0 || h <= 0) {
    return 160;
  }
  return Math.max(72, Math.min(w, h) * 0.52);
});

const sliceCount = computed((): number => props.sliceUrls.length);

/** World-Z gap between slices; caps so tall stacks stay readable. */
const sliceGap = computed((): number => {
  const n = sliceCount.value;
  if (n <= 1) {
    return 0;
  }
  const budget = Math.min(planeSize.value * 0.85, 140);
  return Math.max(6, budget / (n - 1));
});

const hasAnySlice = computed((): boolean =>
  props.sliceUrls.some((url) => url !== null),
);

const canOrbit = computed(
  (): boolean => sliceCount.value > 0 && hasAnySlice.value,
);

function sliceTransform(index: number): string {
  const n = sliceCount.value;
  const mid = (n - 1) / 2;
  const z = (index - mid) * sliceGap.value;
  return `translate(-50%, -50%) translateZ(${String(z)}px)`;
}

function slicePlaneStyle(index: number): {
  width: string;
  height: string;
  transform: string;
} {
  const size = planeSize.value;
  return {
    width: `${String(size)}px`,
    height: `${String(size)}px`,
    transform: sliceTransform(index),
  };
}

function onScenePointerDown(event: PointerEvent): void {
  if (!canOrbit.value) {
    return;
  }
  onPointerDown(event);
}

function onSceneDblClick(): void {
  if (!canOrbit.value) {
    return;
  }
  resetOrbit();
}
</script>

<template>
  <div
    ref="sceneRef"
    class="depth-scene relative min-h-0 flex-1 overflow-hidden rounded-lg border border-transparent bg-surface-sunken shadow-sm select-none"
    :class="canOrbit ? (dragging ? 'cursor-grabbing' : 'cursor-grab') : ''"
    role="img"
    :aria-label="`Depth volume, ${String(sliceCount)} slices, active ${String(activeIndex)}. Drag to orbit, double-click to reset.`"
    @pointerdown="onScenePointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
    @dblclick="onSceneDblClick"
  >
    <div
      v-if="loading && !hasAnySlice"
      class="absolute inset-0 flex items-center justify-center text-sm text-ink-muted"
    >
      Loading…
    </div>
    <div
      v-else-if="sliceCount === 0"
      class="absolute inset-0 flex items-center justify-center p-4 text-sm text-ink-muted"
    >
      {{ emptyLabel }}
    </div>
    <div
      v-else
      class="depth-stage absolute inset-0 flex items-center justify-center"
    >
      <div
        class="depth-volume"
        :style="{ transform: volumeTransform }"
      >
        <div
          v-for="(url, index) in sliceUrls"
          :key="index"
          class="depth-slice"
          :class="{ 'is-active': index === activeIndex }"
          :style="slicePlaneStyle(index)"
        >
          <img
            v-if="url !== null"
            :src="url"
            :alt="`Depth slice ${String(index)}`"
            class="depth-slice-img"
            draggable="false"
            @dragstart.prevent
          >
          <div
            v-else
            class="checkerboard depth-slice-empty"
            aria-hidden="true"
          />
        </div>
      </div>
    </div>
    <span
      class="pointer-events-none absolute bottom-2 left-2 z-10 rounded bg-black/55 px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-white"
    >
      Volume · drag
    </span>
  </div>
</template>

<style lang="scss" scoped>
/* preserve-3d + perspective cannot be expressed as UnoCSS utilities. */
.depth-scene {
  perspective: 920px;
  perspective-origin: 50% 42%;
  touch-action: none;
}

.depth-stage {
  transform-style: preserve-3d;
}

.depth-volume {
  position: relative;
  width: 1px;
  height: 1px;
  transform-style: preserve-3d;
}

.depth-slice {
  position: absolute;
  top: 50%;
  left: 50%;
  margin: 0;
  transform-style: preserve-3d;
  border: 1px solid rgb(0 0 0 / 45%);
  background-color: var(--color-ink);
  backface-visibility: hidden;

  &.is-active {
    border-color: var(--color-brand);
    box-shadow: 0 0 0 2px var(--color-brand);
    z-index: 1;
  }
}

.depth-slice-img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: fill;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
  pointer-events: none;
}

.depth-slice-empty {
  width: 100%;
  height: 100%;
}
</style>
