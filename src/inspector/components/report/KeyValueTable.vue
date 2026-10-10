<script setup lang="ts">
import { computed } from "vue";
import type { KeyValueEntry } from "../../../bindings/KeyValueEntry";
import Panel from "../../../shared/components/Panel.vue";
import { displayKeyValue, displayValue, MISSING } from "../../store";

const props = defineProps<{
  entries: KeyValueEntry[] | null;
}>();

const rows = computed((): KeyValueEntry[] => props.entries ?? []);
</script>

<template>
  <Panel title="Other key/value pairs">
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
              key
            </th>
            <th class="px-2 py-1 font-medium">
              value
            </th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="(entry, index) in rows"
            :key="index"
            class="border-b border-line/60"
          >
            <td class="px-2 py-1 font-mono">
              {{ displayValue(entry.key) }}
            </td>
            <td class="min-w-0 break-words px-2 py-1 font-mono">
              {{ displayKeyValue(entry.value) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </Panel>
</template>
