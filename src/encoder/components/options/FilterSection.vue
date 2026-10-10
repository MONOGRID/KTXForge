<script setup lang="ts">
import type { MipmapFilter } from "../../../bindings/MipmapFilter";
import type { MipmapWrap } from "../../../bindings/MipmapWrap";
import Panel from "../../../shared/components/Panel.vue";
import { store } from "../../store";
import OptionControls from "./OptionControls.vue";
import { isLabeled, type OptionControlField } from "./optionControls";

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
    key: "mipmap-filter",
    label: "Mipmap filter",
    kind: "enum",
    labels: mipmapFilterLabels,
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
    get: () => store.options.filter.mipmapWrap,
    set: assignMipmapWrap,
  },
];
</script>

<template>
  <Panel title="Mipmap / resample filters">
    <OptionControls :fields="fields" />
  </Panel>
</template>
