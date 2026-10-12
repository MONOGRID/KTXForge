<script setup lang="ts">
import type { SelectScrollUpButtonProps } from "reka-ui";
import type { HTMLAttributes } from "vue";
import { reactiveOmit } from "@vueuse/core";
import { ChevronUp } from "@lucide/vue";
import { SelectScrollUpButton, useForwardProps } from "reka-ui";
import { cn } from "@/lib/utils";

const props = defineProps<
  SelectScrollUpButtonProps & { class?: HTMLAttributes["class"] }
>();

const delegatedProps = reactiveOmit(props, "class");

const forwardedProps = useForwardProps(delegatedProps);
</script>

<template>
  <SelectScrollUpButton
    v-bind="forwardedProps"
    :class="cn('flex cursor-default items-center justify-center py-0.5', props.class)"
  >
    <slot>
      <ChevronUp class="h-3 w-3" />
    </slot>
  </SelectScrollUpButton>
</template>
