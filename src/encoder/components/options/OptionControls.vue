<script setup lang="ts">
import { computed, toRef } from "vue";
import OptionField from "../../../shared/components/OptionField.vue";
import { fieldState, isOptionEnabled } from "../../store";
import OptionControlInput from "./OptionControlInput.vue";
import {
  warningsForFields,
  type OptionControlField,
} from "./optionControls";

const props = defineProps<{
  fields: OptionControlField[];
}>();

const fieldsRef = toRef(props, "fields");
const warningMap = warningsForFields(() => fieldsRef.value);

const visibleFields = computed(() =>
  fieldsRef.value.filter((field) => isOptionEnabled(field.key)),
);
</script>

<template>
  <OptionField
    v-for="field in visibleFields"
    :key="field.key"
    :label="field.label"
    :option-key="field.key"
    :state="fieldState(field.key)"
  >
    <OptionControlInput :field="field" />
    <p
      v-for="(warning, index) in warningMap.get(field.key) ?? []"
      :key="`${field.key}-${String(index)}`"
      class="hint"
      :class="{ 'text-danger': warning.level === 'warn' }"
    >
      {{ warning.message }}
    </p>
  </OptionField>
</template>
