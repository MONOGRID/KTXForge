<script setup lang="ts">
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const props = withDefaults(
  defineProps<{
    text: string;
    side?: "top" | "right" | "bottom" | "left";
    class?: string;
    contentClass?: string;
  }>(),
  {
    side: "top",
    class: undefined,
    contentClass: undefined,
  },
);
</script>

<template>
  <Tooltip v-if="text.length > 0">
    <TooltipTrigger
      as-child
      :class="cn('inline-flex max-w-full min-w-0', props.class)"
    >
      <slot />
    </TooltipTrigger>
    <TooltipContent
      :side="side"
      :class="cn('max-w-xs text-pretty', contentClass)"
    >
      {{ text }}
    </TooltipContent>
  </Tooltip>
  <span
    v-else
    :class="cn('inline-flex max-w-full min-w-0', props.class)"
  >
    <slot />
  </span>
</template>
