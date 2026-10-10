<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.2
import type { FieldState } from "../../bindings/FieldState";
import type { OptionKey } from "../../bindings/OptionKey";

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
</script>

<template>
  <div
    class="field-row"
    :class="{ 'opacity-50': state && !state.enabled }"
  >
    <div class="label">
      {{ label }}
    </div>
    <div
      class="min-w-0 flex-1"
      :class="{ 'pointer-events-none': state && !state.enabled }"
    >
      <slot />
      <p
        v-if="state && !state.enabled && state.reason"
        class="hint"
      >
        {{ state.reason }}
      </p>
    </div>
  </div>
</template>
