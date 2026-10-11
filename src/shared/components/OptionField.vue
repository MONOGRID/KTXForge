<script setup lang="ts">
import { computed, provide, useId } from "vue";
import type { FieldState } from "../../bindings/FieldState";
import type { OptionKey } from "../../bindings/OptionKey";
import { helpFor } from "../optionHelp";
import { optionControlIdKey } from "./optionControlId";
import Tooltip from "./Tooltip.vue";

const props = withDefaults(
  defineProps<{
    label: string;
    optionKey: OptionKey;
    state?: FieldState;
    help?: string;
  }>(),
  {
    state: undefined,
    help: undefined,
  },
);

const controlId = useId();
provide(optionControlIdKey, controlId);

/** Hide unless Rust reported the field enabled. No state → hidden. */
const visible = computed((): boolean => props.state?.enabled === true);

const tooltip = computed((): string => props.help ?? helpFor(props.optionKey));
</script>

<template>
  <div
    v-if="visible"
    class="field-row"
    :data-option-key="optionKey"
  >
    <Tooltip
      :text="tooltip"
      class="label"
    >
      <label
        class="min-w-0 cursor-pointer truncate"
        :for="controlId"
        :title="label"
      >{{ label }}</label>
    </Tooltip>
    <div class="min-w-0 flex-1">
      <slot :control-id="controlId" />
    </div>
  </div>
</template>
