<script setup lang="ts">
import Panel from "../../../shared/components/Panel.vue";
import { store } from "../../store";
import OptionControls from "./OptionControls.vue";
import {
  colorPrimariesLabels,
  convertPrimariesLabels,
  convertTfLabels,
  texcoordOriginLabels,
  transferFunctionLabels,
} from "./colorLabels";
import { isLabeled, type OptionControlField } from "./optionControls";

function setEnum<E extends string>(
  labels: Record<E, string>,
  value: string | null,
  assign: (next: E | null) => void,
): void {
  if (value === null) {
    assign(null);
    return;
  }
  if (isLabeled(labels, value)) {
    assign(value);
  }
}

const fields: OptionControlField[] = [
  {
    key: "assign-tf",
    label: "Assign TF",
    kind: "enum",
    labels: transferFunctionLabels,
    get: () => store.options.color.assignTf,
    set: (value) => {
      setEnum(transferFunctionLabels, value, (next) => {
        store.options.color.assignTf = next;
      });
    },
  },
  {
    key: "convert-tf",
    label: "Convert TF",
    kind: "enum",
    labels: convertTfLabels,
    get: () => store.options.color.convertTf,
    set: (value) => {
      setEnum(convertTfLabels, value, (next) => {
        store.options.color.convertTf = next;
      });
    },
  },
  {
    key: "assign-primaries",
    label: "Assign primaries",
    kind: "enum",
    labels: colorPrimariesLabels,
    get: () => store.options.color.assignPrimaries,
    set: (value) => {
      setEnum(colorPrimariesLabels, value, (next) => {
        store.options.color.assignPrimaries = next;
      });
    },
  },
  {
    key: "convert-primaries",
    label: "Convert primaries",
    kind: "enum",
    labels: convertPrimariesLabels,
    get: () => store.options.color.convertPrimaries,
    set: (value) => {
      setEnum(convertPrimariesLabels, value, (next) => {
        store.options.color.convertPrimaries = next;
      });
    },
  },
  {
    key: "assign-texcoord-origin",
    label: "Assign texcoord origin",
    kind: "enum",
    labels: texcoordOriginLabels,
    get: () => store.options.color.assignTexcoordOrigin,
    set: (value) => {
      setEnum(texcoordOriginLabels, value, (next) => {
        store.options.color.assignTexcoordOrigin = next;
      });
    },
  },
  {
    key: "convert-texcoord-origin",
    label: "Convert texcoord origin",
    kind: "enum",
    labels: texcoordOriginLabels,
    get: () => store.options.color.convertTexcoordOrigin,
    set: (value) => {
      setEnum(texcoordOriginLabels, value, (next) => {
        store.options.color.convertTexcoordOrigin = next;
      });
    },
  },
  {
    key: "fail-on-color-conversions",
    label: "Fail on color conversions",
    kind: "bool",
    get: () => store.options.color.failOnColorConversions,
    set: (value) => {
      store.options.color.failOnColorConversions = value;
    },
  },
  {
    key: "warn-on-color-conversions",
    label: "Warn on color conversions",
    kind: "bool",
    get: () => store.options.color.warnOnColorConversions,
    set: (value) => {
      store.options.color.warnOnColorConversions = value;
    },
  },
  {
    key: "no-warn-on-color-conversions",
    label: "No warn on color conversions",
    kind: "bool",
    get: () => store.options.color.noWarnOnColorConversions,
    set: (value) => {
      store.options.color.noWarnOnColorConversions = value;
    },
  },
  {
    key: "fail-on-origin-changes",
    label: "Fail on origin changes",
    kind: "bool",
    get: () => store.options.color.failOnOriginChanges,
    set: (value) => {
      store.options.color.failOnOriginChanges = value;
    },
  },
  {
    key: "warn-on-origin-changes",
    label: "Warn on origin changes",
    kind: "bool",
    get: () => store.options.color.warnOnOriginChanges,
    set: (value) => {
      store.options.color.warnOnOriginChanges = value;
    },
  },
  {
    key: "swizzle",
    label: "Swizzle",
    kind: "text",
    get: () => store.options.color.swizzle,
    set: (value) => {
      store.options.color.swizzle = value;
    },
  },
  {
    key: "input-swizzle",
    label: "Input swizzle",
    kind: "text",
    get: () => store.options.color.inputSwizzle,
    set: (value) => {
      store.options.color.inputSwizzle = value;
    },
  },
  {
    key: "normalize",
    label: "Normalize",
    kind: "bool",
    get: () => store.options.color.normalize,
    set: (value) => {
      store.options.color.normalize = value;
    },
  },
  {
    key: "normal-mode",
    label: "Normal mode",
    kind: "bool",
    get: () => store.options.color.normalMode,
    set: (value) => {
      store.options.color.normalMode = value;
    },
  },
];
</script>

<template>
  <Panel title="Color, orientation, channels">
    <OptionControls :fields="fields" />
  </Panel>
</template>
