<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.5
import { computed } from "vue";
import { store } from "../../store";

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
  const report = store.report;
  if (report === null) {
    return null;
  }

  if (report.kind.kind === "ktx2") {
    const header = report.kind.info.header;
    const anim = report.kind.info.keyValueData.KTXanimData;
    return {
      faceCount: Math.max(1, header.faceCount),
      layerCount: Math.max(1, header.layerCount),
      depthCount: Math.max(1, header.pixelDepth),
      hasAnim: anim !== null,
    };
  }

  const header = report.kind.header;
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

const layerOptions = computed((): number[] => {
  const count = dims.value?.layerCount ?? 0;
  return Array.from({ length: count }, (_, i) => i);
});

const depthOptions = computed((): number[] => {
  const count = dims.value?.depthCount ?? 0;
  return Array.from({ length: count }, (_, i) => i);
});

function selectFace(face: number): void {
  store.selection.face = face;
}

function onLayerChange(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement)) {
    return;
  }
  store.selection.layer = Number(el.value);
}

function onDepthChange(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement)) {
    return;
  }
  store.selection.depthSlice = Number(el.value);
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
          :class="{ 'btn-primary': store.selection.face === index }"
          :aria-pressed="store.selection.face === index"
          :disabled="dims !== null && index >= dims.faceCount"
          @click="selectFace(index)"
        >
          {{ label }}
        </button>
      </div>
    </div>

    <div
      v-if="showLayers"
      class="field-row items-center"
    >
      <label
        class="label pt-0"
        for="inspector-layer"
      >Layer</label>
      <select
        id="inspector-layer"
        class="rounded border border-line bg-surface-raised px-2 py-1 text-sm"
        :value="store.selection.layer"
        @change="onLayerChange"
      >
        <option
          v-for="layer in layerOptions"
          :key="layer"
          :value="layer"
        >
          {{ layer }}
        </option>
      </select>
    </div>

    <div
      v-if="showDepth"
      class="field-row items-center"
    >
      <label
        class="label pt-0"
        for="inspector-depth"
      >Depth</label>
      <select
        id="inspector-depth"
        class="rounded border border-line bg-surface-raised px-2 py-1 text-sm"
        :value="store.selection.depthSlice"
        @change="onDepthChange"
      >
        <option
          v-for="depth in depthOptions"
          :key="depth"
          :value="depth"
        >
          {{ depth }}
        </option>
      </select>
    </div>
  </div>
</template>
