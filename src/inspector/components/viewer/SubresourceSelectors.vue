<script setup lang="ts">
import { computed } from "vue";
import { report, selection } from "../../store";
import RangeTickLabels from "../../../shared/components/RangeTickLabels.vue";

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
  depthCount: number;
  hasAnim: boolean;
};

const dims = computed((): Dims | null => {
  const current = report.value;
  if (current === null) {
    return null;
  }

  if (current.kind.kind === "ktx2") {
    const header = current.kind.info.header;
    const anim = current.kind.info.keyValueData.KTXanimData;
    return {
      faceCount: Math.max(1, header.faceCount),
      layerCount: Math.max(1, header.layerCount),
      depthCount: Math.max(1, header.pixelDepth),
      hasAnim: anim !== null,
    };
  }

  const header = current.kind.header;
  return {
    faceCount: Math.max(1, header.numberOfFaces),
    layerCount: Math.max(1, header.numberOfArrayElements),
    depthCount: Math.max(1, header.pixelDepth),
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
  (): boolean => dims.value !== null && dims.value.depthCount > 1,
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
</script>

<template>
  <div
    v-if="visible"
    class="flex flex-col gap-2"
  >
    <div
      v-if="showFaces"
      class="flex flex-col gap-1"
    >
      <div class="text-sm text-ink-muted">
        Face
      </div>
      <div class="flex flex-wrap gap-1">
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
      class="flex flex-col gap-1"
    >
      <label
        class="text-sm text-ink-muted"
        for="inspector-layer"
      >Layer {{ selection.layer }}</label>
      <input
        id="inspector-layer"
        type="range"
        class="w-full accent-brand"
        min="0"
        :max="layerMax"
        step="1"
        :value="selection.layer"
        @input="onLayerInput"
      >
      <RangeTickLabels
        :min="0"
        :max="layerMax"
      />
    </div>

    <div
      v-if="showDepth"
      class="flex flex-col gap-1"
    >
      <label
        class="text-sm text-ink-muted"
        for="inspector-depth"
      >Depth {{ selection.depthSlice }}</label>
      <input
        id="inspector-depth"
        type="range"
        class="w-full accent-brand"
        min="0"
        :max="depthMax"
        step="1"
        :value="selection.depthSlice"
        @input="onDepthInput"
      >
      <RangeTickLabels
        :min="0"
        :max="depthMax"
      />
    </div>
  </div>
</template>
