<script setup lang="ts">
import { provide, useId } from "vue";
import type { FieldState } from "../../bindings/FieldState";
import type { OptionKey } from "../../bindings/OptionKey";
import { optionControlIdKey } from "./optionControlId";

withDefaults(
  defineProps<{
    label: string;
    optionKey: OptionKey;
    state?: FieldState;
  }>(),
  {
    state: undefined,
  },
);

const controlId = useId();
provide(optionControlIdKey, controlId);
</script>

<template>
  <div
    class="field-row"
    :data-option-key="optionKey"
    :class="{
      'pointer-events-none opacity-50': state && !state.enabled,
    }"
  >
    <label
      class="label"
      :for="controlId"
      :title="label"
    >{{ label }}</label>
    <div class="min-w-0 flex-1">
      <slot :control-id="controlId" />
      <p
        v-if="state && !state.enabled && state.reason"
        class="hint"
      >
        {{ state.reason }}
      </p>
    </div>
  </div>
</template>
