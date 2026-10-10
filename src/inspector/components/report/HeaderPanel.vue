<script setup lang="ts">
import type { Ktx1Header } from "../../../bindings/Ktx1Header";
import type { Ktx2Header } from "../../../bindings/Ktx2Header";
import Panel from "../../../shared/components/Panel.vue";
import { displayValue, MISSING } from "../../store";
import ReportField from "./ReportField.vue";

defineProps<{
  ktx2: Ktx2Header | null;
  ktx1: Ktx1Header | null;
}>();
</script>

<template>
  <Panel title="Header">
    <template v-if="ktx2 !== null">
      <ReportField
        label="identifier"
        :value="displayValue(ktx2.identifier)"
      />
      <ReportField
        label="vkFormat"
        :value="displayValue(ktx2.vkFormat)"
      />
      <ReportField
        label="typeSize"
        :value="displayValue(ktx2.typeSize)"
      />
      <ReportField
        label="pixelWidth"
        :value="displayValue(ktx2.pixelWidth)"
      />
      <ReportField
        label="pixelHeight"
        :value="displayValue(ktx2.pixelHeight)"
      />
      <ReportField
        label="pixelDepth"
        :value="displayValue(ktx2.pixelDepth)"
      />
      <ReportField
        label="layerCount"
        :value="displayValue(ktx2.layerCount)"
      />
      <ReportField
        label="faceCount"
        :value="displayValue(ktx2.faceCount)"
      />
      <ReportField
        label="levelCount"
        :value="displayValue(ktx2.levelCount)"
      />
      <ReportField
        label="supercompressionScheme"
        :value="displayValue(ktx2.supercompressionScheme)"
      />
    </template>
    <template v-else-if="ktx1 !== null">
      <ReportField
        label="littleEndian"
        :value="displayValue(ktx1.littleEndian)"
      />
      <ReportField
        label="glType"
        :value="displayValue(ktx1.glType)"
      />
      <ReportField
        label="glTypeSize"
        :value="displayValue(ktx1.glTypeSize)"
      />
      <ReportField
        label="glFormat"
        :value="displayValue(ktx1.glFormat)"
      />
      <ReportField
        label="glInternalFormat"
        :value="displayValue(ktx1.glInternalFormat)"
      />
      <ReportField
        label="glBaseInternalFormat"
        :value="displayValue(ktx1.glBaseInternalFormat)"
      />
      <ReportField
        label="pixelWidth"
        :value="displayValue(ktx1.pixelWidth)"
      />
      <ReportField
        label="pixelHeight"
        :value="displayValue(ktx1.pixelHeight)"
      />
      <ReportField
        label="pixelDepth"
        :value="displayValue(ktx1.pixelDepth)"
      />
      <ReportField
        label="numberOfArrayElements"
        :value="displayValue(ktx1.numberOfArrayElements)"
      />
      <ReportField
        label="numberOfFaces"
        :value="displayValue(ktx1.numberOfFaces)"
      />
      <ReportField
        label="numberOfMipmapLevels"
        :value="displayValue(ktx1.numberOfMipmapLevels)"
      />
      <ReportField
        label="bytesOfKeyValueData"
        :value="displayValue(ktx1.bytesOfKeyValueData)"
      />
    </template>
    <div
      v-else
      class="py-1.5 text-sm text-ink-muted"
    >
      {{ MISSING }}
    </div>
  </Panel>
</template>
