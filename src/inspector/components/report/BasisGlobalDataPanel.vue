<script setup lang="ts">
import { computed } from "vue";
import type { BasisLzGlobalData } from "../../../bindings/BasisLzGlobalData";
import type { BasisLzImageSlice } from "../../../bindings/BasisLzImageSlice";
import Panel from "../../../shared/components/Panel.vue";
import { displayList, displayValue, MISSING } from "../../store";
import ReportField from "./ReportField.vue";

const props = defineProps<{
  data: BasisLzGlobalData | null;
}>();

const images = computed((): BasisLzImageSlice[] => props.data?.images ?? []);
</script>

<template>
  <Panel title="BasisLZ global data">
    <template v-if="data !== null">
      <ReportField
        label="type"
        :value="displayValue(data.type)"
      />
      <ReportField
        label="endpointCount"
        :value="displayValue(data.endpointCount)"
      />
      <ReportField
        label="selectorCount"
        :value="displayValue(data.selectorCount)"
      />
      <ReportField
        label="endpointsByteLength"
        :value="displayValue(data.endpointsByteLength)"
      />
      <ReportField
        label="selectorsByteLength"
        :value="displayValue(data.selectorsByteLength)"
      />
      <ReportField
        label="tablesByteLength"
        :value="displayValue(data.tablesByteLength)"
      />
      <ReportField
        label="extendedByteLength"
        :value="displayValue(data.extendedByteLength)"
      />
      <div
        v-if="images.length === 0"
        class="mt-2 text-sm text-ink-muted"
      >
        images: {{ MISSING }}
      </div>
      <div
        v-else
        class="mt-2 overflow-x-auto"
      >
        <table class="w-full border-collapse text-left text-sm">
          <thead>
            <tr class="border-b border-line text-ink-muted">
              <th class="px-2 py-1 font-medium">
                #
              </th>
              <th class="px-2 py-1 font-medium">
                imageFlags
              </th>
              <th class="px-2 py-1 font-medium">
                rgbSliceByteOffset
              </th>
              <th class="px-2 py-1 font-medium">
                rgbSliceByteLength
              </th>
              <th class="px-2 py-1 font-medium">
                alphaSliceByteOffset
              </th>
              <th class="px-2 py-1 font-medium">
                alphaSliceByteLength
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(image, index) in images"
              :key="index"
              class="border-b border-line/60 font-mono"
            >
              <td class="px-2 py-1">
                {{ index }}
              </td>
              <td class="px-2 py-1">
                {{ displayList(image.imageFlags) }}
              </td>
              <td class="px-2 py-1">
                {{ displayValue(image.rgbSliceByteOffset) }}
              </td>
              <td class="px-2 py-1">
                {{ displayValue(image.rgbSliceByteLength) }}
              </td>
              <td class="px-2 py-1">
                {{ displayValue(image.alphaSliceByteOffset) }}
              </td>
              <td class="px-2 py-1">
                {{ displayValue(image.alphaSliceByteLength) }}
              </td>
            </tr>
          </tbody>
        </table>
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
