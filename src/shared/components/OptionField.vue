<script setup lang="ts">
import { computed, provide, useId } from "vue";
import type { FieldState } from "../../bindings/FieldState";
import type { OptionKey } from "../../bindings/OptionKey";
import { optionControlIdKey } from "./optionControlId";
import HelpTooltip from "./HelpTooltip.vue";

const props = withDefaults(
  defineProps<{
    label: string;
    optionKey: OptionKey;
    state?: FieldState;
    /** Override Rust help (rare; prefer `state.help`). */
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

const tooltip = computed((): string => props.help ?? props.state?.help ?? "");
</script>

<template>
  <div
    v-if="visible"
    class="field-row"
    :data-option-key="optionKey"
  >
    <HelpTooltip
      :text="tooltip"
      class="label"
    >
      <label
        class="min-w-0 cursor-pointer truncate"
        :for="controlId"
        :title="label"
      >{{ label }}</label>
    </HelpTooltip>
    <div class="min-w-0 flex-1">
      <slot :control-id="controlId" />
    </div>
  </div>
</template>
