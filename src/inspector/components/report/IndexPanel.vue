<script setup lang="ts">
import type { ByteRange } from "../../../bindings/ByteRange";
import Panel from "../../../shared/components/Panel.vue";
import { displayValue, MISSING } from "../../store";
import ReportField from "./ReportField.vue";

defineProps<{
  dataFormatDescriptor: ByteRange | null;
  keyValueData: ByteRange | null;
  supercompressionGlobalData: ByteRange | null;
}>();
</script>

<template>
  <Panel title="Index offsets">
    <template v-if="dataFormatDescriptor !== null">
      <div class="mb-2 text-xs font-medium text-ink-muted">
        dataFormatDescriptor
      </div>
      <ReportField
        label="byteOffset"
        :value="displayValue(dataFormatDescriptor.byteOffset)"
      />
      <ReportField
        label="byteLength"
        :value="displayValue(dataFormatDescriptor.byteLength)"
      />
      <div class="mb-2 mt-3 text-xs font-medium text-ink-muted">
        keyValueData
      </div>
      <ReportField
        label="byteOffset"
        :value="displayValue(keyValueData?.byteOffset)"
      />
      <ReportField
        label="byteLength"
        :value="displayValue(keyValueData?.byteLength)"
      />
      <div class="mb-2 mt-3 text-xs font-medium text-ink-muted">
        supercompressionGlobalData
      </div>
      <ReportField
        label="byteOffset"
        :value="displayValue(supercompressionGlobalData?.byteOffset)"
      />
      <ReportField
        label="byteLength"
        :value="displayValue(supercompressionGlobalData?.byteLength)"
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
