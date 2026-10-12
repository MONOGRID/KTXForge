<script setup lang="ts">
import { FileText } from "@lucide/vue";
import Panel from "../../../shared/components/Panel.vue";
import { MISSING } from "../../store";

defineProps<{
  text: string | null;
}>();
</script>

<template>
  <Panel
    title="KTX1 text"
    :icon="FileText"
  >
    <pre
      v-if="text !== null && text.length > 0"
      class="m-0 max-h-96 overflow-auto whitespace-pre-wrap break-words font-mono text-xs text-ink"
    >{{ text }}</pre>
    <div
      v-else
      class="py-1.5 text-sm text-ink-muted"
    >
      {{ MISSING }}
    </div>
  </Panel>
</template>
