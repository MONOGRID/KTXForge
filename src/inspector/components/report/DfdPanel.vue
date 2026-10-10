<script setup lang="ts">
import { computed } from "vue";
import type { DataFormatDescriptor } from "../../../bindings/DataFormatDescriptor";
import type { DfdBlock } from "../../../bindings/DfdBlock";
import Panel from "../../../shared/components/Panel.vue";
import {
  displayList,
  displayNumbers,
  displayValue,
  MISSING,
} from "../../store";
import ReportField from "./ReportField.vue";

const props = defineProps<{
  descriptor: DataFormatDescriptor | null;
}>();

const blocks = computed((): DfdBlock[] => props.descriptor?.blocks ?? []);
</script>

<template>
  <Panel title="Data format descriptor">
    <template v-if="descriptor !== null">
      <ReportField
        label="totalSize"
        :value="displayValue(descriptor.totalSize)"
      />
      <div
        v-if="blocks.length === 0"
        class="py-1.5 text-sm text-ink-muted"
      >
        {{ MISSING }}
      </div>
      <div
        v-for="(block, blockIndex) in blocks"
        :key="blockIndex"
        class="mt-3 rounded border border-line p-2"
      >
        <div class="mb-1 text-xs font-medium text-ink-muted">
          Block {{ blockIndex }}
        </div>
        <ReportField
          label="vendor"
          :value="displayValue(block.vendorId)"
        />
        <ReportField
          label="descriptorType"
          :value="displayValue(block.descriptorType)"
        />
        <ReportField
          label="version"
          :value="displayValue(block.versionNumber)"
        />
        <ReportField
          label="descriptorBlockSize"
          :value="displayValue(block.descriptorBlockSize)"
        />
        <ReportField
          label="flags"
          :value="displayList(block.flags)"
        />
        <ReportField
          label="transferFunction"
          :value="displayValue(block.transferFunction)"
        />
        <ReportField
          label="colorPrimaries"
          :value="displayValue(block.colorPrimaries)"
        />
        <ReportField
          label="colorModel"
          :value="displayValue(block.colorModel)"
        />
        <ReportField
          label="texelBlockDimension"
          :value="displayNumbers(block.texelBlockDimension)"
        />
        <ReportField
          label="bytesPerPlane"
          :value="displayNumbers(block.bytesPlane)"
        />
        <div
          v-if="block.samples.length === 0"
          class="mt-2 text-sm text-ink-muted"
        >
          samples: {{ MISSING }}
        </div>
        <div
          v-else
          class="mt-2 overflow-x-auto"
        >
          <table class="w-full border-collapse text-left text-sm">
            <thead>
              <tr class="border-b border-line text-ink-muted">
                <th class="px-2 py-1 font-medium">
                  channel
                </th>
                <th class="px-2 py-1 font-medium">
                  qualifiers
                </th>
                <th class="px-2 py-1 font-medium">
                  bitOffset
                </th>
                <th class="px-2 py-1 font-medium">
                  bitLength
                </th>
                <th class="px-2 py-1 font-medium">
                  samplePosition
                </th>
                <th class="px-2 py-1 font-medium">
                  sampleLower
                </th>
                <th class="px-2 py-1 font-medium">
                  sampleUpper
                </th>
              </tr>
            </thead>
            <tbody>
              <tr
                v-for="(sample, sampleIndex) in block.samples"
                :key="sampleIndex"
                class="border-b border-line/60 font-mono"
              >
                <td class="px-2 py-1">
                  {{ displayValue(sample.channelType) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayList(sample.qualifiers) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayValue(sample.bitOffset) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayValue(sample.bitLength) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayNumbers(sample.samplePosition) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayValue(sample.sampleLower) }}
                </td>
                <td class="px-2 py-1">
                  {{ displayValue(sample.sampleUpper) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
    <div
      v-else
      class="py-1.5 text-sm text-ink-muted"
    >
      {{ MISSING }}
    </div>
  </Panel>
</template>
