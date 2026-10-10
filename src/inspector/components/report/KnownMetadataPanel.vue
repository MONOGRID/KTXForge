<script setup lang="ts">
import { computed } from "vue";
import type { KeyValueData } from "../../../bindings/KeyValueData";
import Panel from "../../../shared/components/Panel.vue";
import { displayValue, MISSING } from "../../store";
import ReportField from "./ReportField.vue";

const props = defineProps<{
  keyValueData: KeyValueData | null;
}>();

const faceLabels = [
  ["px", "+X"],
  ["nx", "−X"],
  ["py", "+Y"],
  ["ny", "−Y"],
  ["pz", "+Z"],
  ["nz", "−Z"],
] as const;

const cubemapFaces = computed((): string => {
  const cube = props.keyValueData?.KTXcubemapIncomplete;
  if (cube === null || cube === undefined) {
    return MISSING;
  }
  return faceLabels
    .map(([key, label]) => `${label}: ${displayValue(cube[key])}`)
    .join("; ");
});
</script>

<template>
  <Panel title="Known metadata">
    <template v-if="keyValueData !== null">
      <ReportField
        label="KTXorientation"
        :value="displayValue(keyValueData.KTXorientation)"
      />
      <ReportField
        label="KTXswizzle"
        :value="displayValue(keyValueData.KTXswizzle)"
      />
      <ReportField
        label="KTXwriter"
        :value="displayValue(keyValueData.KTXwriter)"
      />
      <ReportField
        label="KTXwriterScParams"
        :value="displayValue(keyValueData.KTXwriterScParams)"
      />
      <ReportField
        label="KTXastcDecodeMode"
        :value="displayValue(keyValueData.KTXastcDecodeMode)"
      />
      <div class="mb-1 mt-2 text-xs font-medium text-ink-muted">
        KTXglFormat
      </div>
      <template v-if="keyValueData.KTXglFormat !== null">
        <ReportField
          label="internalFormat"
          :value="displayValue(keyValueData.KTXglFormat.internalFormat)"
        />
        <ReportField
          label="format"
          :value="displayValue(keyValueData.KTXglFormat.format)"
        />
        <ReportField
          label="type"
          :value="displayValue(keyValueData.KTXglFormat.type)"
        />
      </template>
      <ReportField
        v-else
        label="KTXglFormat"
        :value="MISSING"
      />
      <ReportField
        label="KTXdxgiFormat__"
        :value="displayValue(keyValueData.KTXdxgiFormat__)"
      />
      <ReportField
        label="KTXmetalPixelFormat"
        :value="displayValue(keyValueData.KTXmetalPixelFormat)"
      />
      <ReportField
        label="KTXcubemapIncomplete"
        :value="cubemapFaces"
      />
      <div class="mb-1 mt-2 text-xs font-medium text-ink-muted">
        KTXanimData
      </div>
      <template v-if="keyValueData.KTXanimData !== null">
        <ReportField
          label="duration"
          :value="displayValue(keyValueData.KTXanimData.duration)"
        />
        <ReportField
          label="timescale"
          :value="displayValue(keyValueData.KTXanimData.timescale)"
        />
        <ReportField
          label="loopCount"
          :value="displayValue(keyValueData.KTXanimData.loopCount)"
        />
      </template>
      <ReportField
        v-else
        label="KTXanimData"
        :value="MISSING"
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
