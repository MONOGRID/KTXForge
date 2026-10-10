<script setup lang="ts">
import { computed } from "vue";
import type { ValidationMessage } from "../../../bindings/ValidationMessage";
import Panel from "../../../shared/components/Panel.vue";
import { displayValue, MISSING } from "../../store";
import ReportField from "./ReportField.vue";

const props = defineProps<{
  valid: boolean | null;
  messages: ValidationMessage[] | null;
}>();

const validLabel = computed((): string => {
  if (props.valid === null) {
    return MISSING;
  }
  return props.valid ? "true" : "false";
});

const rows = computed((): ValidationMessage[] => props.messages ?? []);
</script>

<template>
  <Panel title="Status">
    <ReportField
      label="valid"
      :value="validLabel"
    />
    <div
      v-if="rows.length === 0"
      class="py-1.5 text-sm text-ink-muted"
    >
      {{ MISSING }}
    </div>
    <div
      v-for="(msg, index) in rows"
      :key="index"
      class="mt-2 rounded border border-line bg-surface-sunken p-2"
    >
      <ReportField
        label="level"
        :value="displayValue(msg.level)"
      />
      <ReportField
        label="id"
        :value="displayValue(msg.id)"
      />
      <ReportField
        label="message"
        :value="displayValue(msg.message)"
      />
      <ReportField
        label="details"
        :value="displayValue(msg.details)"
      />
    </div>
  </Panel>
</template>
