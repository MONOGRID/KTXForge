<script setup lang="ts">
import { ListOrdered } from "@lucide/vue";
import { computed } from "vue";
import type { LevelIndexEntry } from "../../../bindings/LevelIndexEntry";
import Panel from "../../../shared/components/Panel.vue";
import { displayValue, MISSING } from "../../store";

const props = defineProps<{
  levels: LevelIndexEntry[] | null;
}>();

const rows = computed((): LevelIndexEntry[] => props.levels ?? []);
</script>

<template>
  <Panel
    title="Level index"
    :icon="ListOrdered"
  >
    <div
      v-if="rows.length === 0"
      class="py-1.5 text-sm text-ink-muted"
    >
      {{ MISSING }}
    </div>
    <div
      v-else
      class="overflow-x-auto"
    >
      <table class="w-full border-collapse text-left text-sm">
        <thead>
          <tr class="border-b border-line text-ink-muted">
            <th class="px-2 py-1 font-medium">
              mip
            </th>
            <th class="px-2 py-1 font-medium">
              byteOffset
            </th>
            <th class="px-2 py-1 font-medium">
              byteLength
            </th>
            <th class="px-2 py-1 font-medium">
              uncompressedByteLength
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(level, index) in rows"
            :key="index"
            class="border-b border-line/60 font-mono"
          >
            <td class="px-2 py-1">
              {{ index }}
            </td>
            <td class="px-2 py-1">
              {{ displayValue(level.byteOffset) }}
            </td>
            <td class="px-2 py-1">
              {{ displayValue(level.byteLength) }}
            </td>
            <td class="px-2 py-1">
              {{ displayValue(level.uncompressedByteLength) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Panel>
</template>
