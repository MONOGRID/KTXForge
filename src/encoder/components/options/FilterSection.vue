<script setup lang="ts">
import type { MipmapFilter } from "../../../bindings/MipmapFilter";
import type { MipmapWrap } from "../../../bindings/MipmapWrap";
import type { OptionKey } from "../../../bindings/OptionKey";
import Panel from "../../../shared/components/Panel.vue";
import { store } from "../../store";
import OptionControls from "./OptionControls.vue";
import { anyEnabled, isLabeled, type OptionControlField } from "./optionControls";

const KEYS = [
  "generate-mipmap",
  "runtime-mipmap",
  "mipmap-filter",
  "mipmap-filter-scale",
  "mipmap-wrap",
] as const satisfies readonly OptionKey[];

const mipmapFilterLabels = {
  box: "box",
  tent: "tent",
  bell: "bell",
  "b-spline": "b-spline",
  mitchell: "mitchell",
  blackman: "blackman",
  lanczos3: "lanczos3",
  lanczos4: "lanczos4",
  lanczos6: "lanczos6",
  lanczos12: "lanczos12",
  kaiser: "kaiser",
  gaussian: "gaussian",
  catmullrom: "catmullrom",
  quadratic_interp: "quadratic_interp",
  quadratic_approx: "quadratic_approx",
  quadratic_mix: "quadratic_mix",
} satisfies Record<MipmapFilter, string>;

const mipmapWrapLabels = {
  wrap: "wrap",
  reflect: "reflect",
  clamp: "clamp",
} satisfies Record<MipmapWrap, string>;

function assignMipmapFilter(value: string | null): void {
  if (value === null) {
    store.options.filter.mipmapFilter = null;
    return;
  }
  if (isLabeled(mipmapFilterLabels, value)) {
    store.options.filter.mipmapFilter = value;
  }
}

function assignMipmapWrap(value: string | null): void {
  if (value === null) {
    store.options.filter.mipmapWrap = null;
    return;
  }
  if (isLabeled(mipmapWrapLabels, value)) {
    store.options.filter.mipmapWrap = value;
  }
}

const fields: OptionControlField[] = [
  {
    key: "generate-mipmap",
    label: "Generate mipmaps",
    kind: "bool",
    get: () => store.options.container.generateMipmap,
    set: (value) => {
      store.options.container.generateMipmap = value;
    },
  },
  {
    key: "runtime-mipmap",
    label: "Runtime mipmap",
    kind: "bool",
    get: () => store.options.container.runtimeMipmap,
    set: (value) => {
      store.options.container.runtimeMipmap = value;
    },
  },
  {
    key: "mipmap-filter",
    label: "Mipmap filter",
    kind: "enum",
    labels: mipmapFilterLabels,
    emptyLabel: false,
    get: () => store.options.filter.mipmapFilter,
    set: assignMipmapFilter,
  },
  {
    key: "mipmap-filter-scale",
    label: "Mipmap filter scale",
    kind: "number",
    step: "any",
    get: () => store.options.filter.mipmapFilterScale,
    set: (value) => {
      store.options.filter.mipmapFilterScale = value;
    },
  },
  {
    key: "mipmap-wrap",
    label: "Mipmap wrap",
    kind: "enum",
    labels: mipmapWrapLabels,
    emptyLabel: false,
    get: () => store.options.filter.mipmapWrap,
    set: assignMipmapWrap,
  },
];
</script>

<template>
  <Panel
    v-if="anyEnabled(KEYS)"
    title="Mipmaps / resample"
  >
    <OptionControls :fields="fields" />
  </Panel>
</template>
