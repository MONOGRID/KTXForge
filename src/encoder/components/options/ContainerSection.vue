<script setup lang="ts">
import type { OptionKey } from "../../../bindings/OptionKey";
import Panel from "../../../shared/components/Panel.vue";
import { store } from "../../store";
import OptionControls from "./OptionControls.vue";
import { anyEnabled, type OptionControlField } from "./optionControls";

/** Arity fields (layers / cubemap / depth) are owned by the Inputs role panel. */
const KEYS = [
  "1d",
  "levels",
  "width",
  "height",
  "scale",
  "raw",
] as const satisfies readonly OptionKey[];

const fields: OptionControlField[] = [
  {
    key: "1d",
    label: "1D",
    kind: "bool",
    get: () => store.options.container.oneD,
    set: (value) => {
      store.options.container.oneD = value;
    },
  },
  {
    key: "levels",
    label: "Levels",
    kind: "number",
    step: "1",
    get: () => store.options.container.levels,
    set: (value) => {
      store.options.container.levels = value;
    },
  },
  {
    key: "width",
    label: "Width",
    kind: "number",
    step: "1",
    get: () => store.options.container.width,
    set: (value) => {
      store.options.container.width = value;
    },
  },
  {
    key: "height",
    label: "Height",
    kind: "number",
    step: "1",
    get: () => store.options.container.height,
    set: (value) => {
      store.options.container.height = value;
    },
  },
  {
    key: "scale",
    label: "Scale",
    kind: "number",
    step: "any",
    get: () => store.options.container.scale,
    set: (value) => {
      store.options.container.scale = value;
    },
  },
  {
    key: "raw",
    label: "Raw",
    kind: "bool",
    get: () => store.options.container.raw,
    set: (value) => {
      store.options.container.raw = value;
    },
  },
];
</script>

<template>
  <Panel
    v-if="anyEnabled(KEYS)"
    title="Container"
  >
    <OptionControls :fields="fields" />
  </Panel>
</template>
