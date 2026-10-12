<script setup lang="ts">
defineProps<{
  src: string;
  compareSrc: string;
  alt: string;
  compareAlt: string;
  splitPct: number;
  transformStyle: { transform: string };
  canvasStyle: { width: string; height: string } | undefined;
  revealInnerStyle: { width: string };
  originalLabel: string;
  resultLabel: string;
  showChrome: boolean;
}>();

const emit = defineEmits<{
  primaryLoad: [event: Event];
  compareLoad: [event: Event];
  splitPointerDown: [event: PointerEvent];
  splitPointerMove: [event: PointerEvent];
  splitPointerUp: [event: PointerEvent];
}>();
</script>

<template>
  <!--
    Compare wipe in *frame* space: both layers share the same transform, but the
    original is revealed through a left strip whose width is splitPct of the frame.
  -->
  <div class="absolute inset-0 flex items-center justify-center overflow-hidden">
    <div
      class="will-change-transform"
      :style="transformStyle"
    >
      <div
        class="checkerboard relative"
        :style="canvasStyle"
      >
        <img
          :src="compareSrc"
          :alt="compareAlt"
          class="compare-img"
          draggable="false"
          @load="emit('compareLoad', $event)"
          @dragstart.prevent
        >
      </div>
    </div>
  </div>
  <div
    class="absolute inset-y-0 left-0 overflow-hidden"
    :style="{ width: `${String(splitPct)}%` }"
  >
    <div
      class="absolute top-0 left-0 flex h-full items-center justify-center"
      :style="revealInnerStyle"
    >
      <div
        class="will-change-transform"
        :style="transformStyle"
      >
        <div
          class="checkerboard relative"
          :style="canvasStyle"
        >
          <img
            :src="src"
            :alt="alt"
            class="compare-img"
            draggable="false"
            @load="emit('primaryLoad', $event)"
            @dragstart.prevent
          >
        </div>
      </div>
    </div>
  </div>

  <template v-if="showChrome">
    <div
      class="pointer-events-none absolute inset-y-0 z-10 w-px bg-white/90 shadow-[0_0_0_1px_rgba(0,0,0,0.35)]"
      :style="{ left: `${String(splitPct)}%` }"
    />
    <button
      type="button"
      class="absolute top-1/2 z-20 flex h-10 w-4 -translate-x-1/2 -translate-y-1/2 cursor-col-resize items-center justify-center rounded-md border border-transparent bg-surface-raised shadow-md"
      :style="{ left: `${String(splitPct)}%` }"
      :aria-label="`Compare slider, ${String(Math.round(splitPct))} percent original`"
      @pointerdown="emit('splitPointerDown', $event)"
      @pointermove="emit('splitPointerMove', $event)"
      @pointerup="emit('splitPointerUp', $event)"
      @pointercancel="emit('splitPointerUp', $event)"
    >
      <span class="h-5 w-0.5 rounded bg-ink-muted" />
    </button>
    <span
      class="pointer-events-none absolute bottom-2 left-2 z-10 rounded bg-black/55 px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-white"
    >
      {{ originalLabel }}
    </span>
    <span
      class="pointer-events-none absolute bottom-2 right-2 z-10 rounded bg-black/55 px-1.5 py-0.5 text-[10px] font-medium tracking-wide text-white"
    >
      {{ resultLabel }}
    </span>
  </template>
</template>

<style lang="scss" scoped>
.compare-img {
  position: absolute;
  inset: 0;
  display: block;
  width: 100%;
  height: 100%;
  max-width: none;
  object-fit: fill;
  image-rendering: pixelated;
  image-rendering: crisp-edges;
}
</style>
