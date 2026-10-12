<script setup lang="ts">
import { computed, watch } from "vue";
import { report, selection } from "../../store";

const FACE_LABELS = [
  "+X",
  "−X",
  "+Y",
  "−Y",
  "+Z",
  "−Z",
] as const;

type Dims = {
  faceCount: number;
  layerCount: number;
  /** Base-level depth (texture is 3D when > 1). */
  baseDepthCount: number;
  /** Depth at the selected mip (halves each level). */
  depthCount: number;
  hasAnim: boolean;
};

const dims = computed((): Dims | null => {
  const current = report.value;
  if (current === null) {
    return null;
  }

  const level = selection.value.level;

  if (current.kind.kind === "ktx2") {
    const header = current.kind.info.header;
    const anim = current.kind.info.keyValueData.KTXanimData;
    const baseDepthCount = Math.max(1, header.pixelDepth);
    return {
      faceCount: Math.max(1, header.faceCount),
      layerCount: Math.max(1, header.layerCount),
      baseDepthCount,
      depthCount: Math.max(1, baseDepthCount >> level),
      hasAnim: anim !== null,
    };
  }

  const header = current.kind.header;
  const baseDepthCount = Math.max(1, header.pixelDepth);
  return {
    faceCount: Math.max(1, header.numberOfFaces),
    layerCount: Math.max(1, header.numberOfArrayElements),
    baseDepthCount,
    depthCount: Math.max(1, baseDepthCount >> level),
    hasAnim: false,
  };
});

const showFaces = computed(
  (): boolean => dims.value !== null && dims.value.faceCount > 1,
);
const showLayers = computed(
  (): boolean =>
    dims.value !== null && dims.value.layerCount > 1 && !dims.value.hasAnim,
);
const showDepth = computed(
  (): boolean => dims.value !== null && dims.value.baseDepthCount > 1,
);
const visible = computed(
  (): boolean => showFaces.value || showLayers.value || showDepth.value,
);

const layerMax = computed((): number =>
  Math.max(0, (dims.value?.layerCount ?? 1) - 1),
);

const depthMax = computed((): number =>
  Math.max(0, (dims.value?.depthCount ?? 1) - 1),
);

function selectFace(face: number): void {
  selection.value.face = face;
}

function onLayerInput(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  selection.value.layer = Number(el.value);
}

function onDepthInput(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  selection.value.depthSlice = Number(el.value);
}

watch(
  () => dims.value?.depthCount,
  (count) => {
    if (count === undefined) {
      return;
    }
    if (selection.value.depthSlice >= count) {
      selection.value.depthSlice = Math.max(0, count - 1);
    }
  },
);
</script>

<template>
  <div
    v-if="visible"
    class="flex flex-col gap-1"
  >
    <div
      v-if="showFaces"
      class="flex items-center gap-2"
    >
      <span class="shrink-0 whitespace-nowrap text-xs text-ink-muted">
        Face
      </span>
      <div class="flex min-w-0 flex-wrap gap-1">
        <button
          v-for="(label, index) in FACE_LABELS"
          :key="label"
          type="button"
          class="btn font-mono text-xs"
          :class="{ 'btn-primary': selection.face === index }"
          :aria-pressed="selection.face === index"
          :disabled="dims !== null && index >= dims.faceCount"
          @click="selectFace(index)"
        >
          {{ label }}
        </button>
      </div>
    </div>

    <div
      v-if="showLayers"
      class="flex items-center gap-2"
    >
      <label
        class="shrink-0 whitespace-nowrap text-xs text-ink-muted"
        for="inspector-layer"
      >Layer {{ selection.layer }}</label>
      <input
        id="inspector-layer"
        type="range"
        class="h-4 min-w-0 flex-1 accent-brand"
        min="0"
        :max="layerMax"
        step="1"
        :value="selection.layer"
        :aria-valuemin="0"
        :aria-valuemax="layerMax"
        :aria-valuenow="selection.layer"
        @input="onLayerInput"
      >
      <span class="w-7 shrink-0 text-right font-mono text-[10px] tabular-nums text-ink-faint">
        {{ layerMax }}
      </span>
    </div>

    <div
      v-if="showDepth"
      class="flex items-center gap-2"
    >
      <label
        class="shrink-0 whitespace-nowrap text-xs text-ink-muted"
        for="inspector-depth"
      >Depth {{ selection.depthSlice }}</label>
      <input
        id="inspector-depth"
        type="range"
        class="h-4 min-w-0 flex-1 accent-brand"
        min="0"
        :max="depthMax"
        step="1"
        :value="selection.depthSlice"
        :aria-valuemin="0"
        :aria-valuemax="depthMax"
        :aria-valuenow="selection.depthSlice"
        @input="onDepthInput"
      >
      <span class="w-7 shrink-0 text-right font-mono text-[10px] tabular-nums text-ink-faint">
        {{ depthMax }}
      </span>
    </div>
  </div>
</template>
