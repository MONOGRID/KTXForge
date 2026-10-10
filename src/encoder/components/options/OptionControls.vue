<script setup lang="ts">
import { toRef } from "vue";
import OptionField from "../../../shared/components/OptionField.vue";
import { fieldState } from "../../store";
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
</script>

<template>
  <OptionField
    v-for="field in fields"
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
