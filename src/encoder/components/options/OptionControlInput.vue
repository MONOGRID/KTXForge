<script setup lang="ts">
import {
  checkboxChecked,
  enumEntries,
  onEnumChange,
  onNumberChange,
  onTextChange,
  type OptionControlField,
} from "./optionControls";

defineProps<{
  field: OptionControlField;
}>();
</script>

<template>
  <select
    v-if="field.kind === 'enum'"
    class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
    :value="field.get() ?? ''"
    @change="onEnumChange(field, $event)"
  >
    <option value="">
      Default
    </option>
    <option
      v-for="option in enumEntries(field.labels)"
      :key="option.value"
      :value="option.value"
    >
      {{ option.label }}
    </option>
  </select>
  <input
    v-else-if="field.kind === 'number'"
    type="number"
    class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
    :step="field.step"
    :value="field.get() ?? ''"
    @change="onNumberChange(field, $event)"
  >
  <input
    v-else-if="field.kind === 'text'"
    type="text"
    maxlength="4"
    spellcheck="false"
    placeholder="rgba"
    class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
    :value="field.get() ?? ''"
    @change="onTextChange(field, $event)"
  >
  <input
    v-else
    type="checkbox"
    class="mt-1"
    :checked="field.get()"
    @change="field.set(checkboxChecked($event))"
  >
</template>
