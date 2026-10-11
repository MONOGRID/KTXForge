<script setup lang="ts">
withDefaults(
  defineProps<{
    text: string;
    position?: "top" | "bottom";
  }>(),
  {
    position: "top",
  },
);
</script>

<template>
  <span class="tooltip-root relative inline-flex max-w-full">
    <slot />
    <span
      class="tooltip-bubble pointer-events-none absolute z-10 max-w-xs rounded bg-ink px-2 py-1 text-xs text-surface opacity-0 shadow"
      :class="position === 'top' ? 'bottom-full left-1/2 mb-1 -translate-x-1/2' : 'top-full left-1/2 mt-1 -translate-x-1/2'"
      role="tooltip"
    >
      {{ text }}
    </span>
  </span>
</template>

<style lang="scss" scoped>
.tooltip-root:hover .tooltip-bubble,
.tooltip-root:focus-within .tooltip-bubble {
  opacity: 1;
}
</style>
