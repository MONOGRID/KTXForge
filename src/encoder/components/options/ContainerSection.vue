<script setup lang="ts">
import Panel from "../../../shared/components/Panel.vue";
import { store } from "../../store";
import OptionControls from "./OptionControls.vue";
import type { OptionControlField } from "./optionControls";

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
    key: "cubemap",
    label: "Cubemap",
    kind: "bool",
    get: () => store.options.container.cubemap,
    set: (value) => {
      store.options.container.cubemap = value;
    },
  },
  {
    key: "layers",
    label: "Layers",
    kind: "number",
    step: "1",
    get: () => store.options.container.layers,
    set: (value) => {
      store.options.container.layers = value;
    },
  },
  {
    key: "depth",
    label: "Depth",
    kind: "number",
    step: "1",
    get: () => store.options.container.depth,
    set: (value) => {
      store.options.container.depth = value;
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
    key: "generate-mipmap",
    label: "Generate mipmap",
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
  <Panel title="Container">
    <OptionControls :fields="fields" />
  </Panel>
</template>
