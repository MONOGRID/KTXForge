<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from "reka-ui";
import type { HTMLAttributes } from "vue";
import { computed } from "vue";
import { reactiveOmit } from "@vueuse/core";
import {
  SwitchRoot,
  SwitchThumb,
  useForwardPropsEmits,
} from "reka-ui";
import { cn } from "@/lib/utils";

const props = defineProps<
  SwitchRootProps & {
    class?: HTMLAttributes["class"];
    /** shadcn-style alias for `modelValue` (reka-ui's controlled prop). */
    checked?: boolean;
  }
>();

const emits = defineEmits<
  SwitchRootEmits & {
    "update:checked": [value: boolean];
  }
>();

const delegatedProps = reactiveOmit(props, "class", "checked");

const forwarded = useForwardPropsEmits(delegatedProps, emits);

const model = computed({
  // Vue types optional boolean props as `boolean` (absent → false), not `boolean | undefined`.
  get: (): boolean => props.checked || props.modelValue === true,
  set: (value: boolean) => {
    emits("update:modelValue", value);
    emits("update:checked", value);
  },
});
</script>

<template>
  <SwitchRoot
    v-bind="forwarded"
    v-model="model"
    :class="cn(
      'peer inline-flex h-3.5 w-6 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:bg-primary data-[state=unchecked]:bg-input',
      props.class,
    )"
  >
    <SwitchThumb
      :class="cn('pointer-events-none block h-2.5 w-2.5 rounded-full bg-white shadow-md ring-0 transition-transform data-[state=checked]:translate-x-2.5 data-[state=unchecked]:translate-x-0')"
    >
      <slot name="thumb" />
    </SwitchThumb>
  </SwitchRoot>
</template>
