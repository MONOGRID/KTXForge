<script setup lang="ts">
import { computed, inject } from "vue";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import EnumSelect from "../../../shared/components/EnumSelect.vue";
import { optionControlIdKey } from "../../../shared/components/optionControlId";
import {
  enumEntries,
  parseNumberField,
  type OptionControlField,
} from "./optionControls";

const props = defineProps<{
  field: OptionControlField;
}>();

const controlId = inject(optionControlIdKey, undefined);

const enumOptions = computed(() => {
  if (props.field.kind !== "enum") {
    return [];
  }
  return enumEntries(props.field.labels);
});

const numberDisplay = computed((): string => {
  if (props.field.kind !== "number") {
    return "";
  }
  const value = props.field.get();
  return value === null ? "" : String(value);
});

const textDisplay = computed((): string => {
  if (props.field.kind !== "text") {
    return "";
  }
  return props.field.get() ?? "";
});

function onEnumUpdate(value: string | null): void {
  if (props.field.kind === "enum") {
    props.field.set(value);
  }
}

function onNumberUpdate(value: string | number): void {
  if (props.field.kind !== "number") {
    return;
  }
  props.field.set(parseNumberField(String(value), props.field.step === "1"));
}

function onTextUpdate(value: string | number): void {
  if (props.field.kind !== "text") {
    return;
  }
  const raw = String(value).trim();
  props.field.set(raw === "" ? null : raw);
}

function onBoolUpdate(checked: boolean): void {
  if (props.field.kind === "bool") {
    props.field.set(checked);
  }
}
</script>

<template>
  <EnumSelect
    v-if="field.kind === 'enum'"
    :model-value="field.get()"
    :options="enumOptions"
    @update:model-value="onEnumUpdate"
  />
  <Input
    v-else-if="field.kind === 'number'"
    :id="controlId"
    type="number"
    class="h-8 font-mono"
    :step="field.step"
    :model-value="numberDisplay"
    @update:model-value="onNumberUpdate"
  />
  <Input
    v-else-if="field.kind === 'text'"
    :id="controlId"
    type="text"
    maxlength="4"
    spellcheck="false"
    placeholder="rgba"
    class="h-8 font-mono"
    :model-value="textDisplay"
    @update:model-value="onTextUpdate"
  />
  <Switch
    v-else
    :id="controlId"
    class="mt-0.5"
    :checked="field.get()"
    @update:checked="onBoolUpdate"
  />
</template>
