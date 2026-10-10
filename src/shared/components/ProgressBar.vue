<script setup lang="ts">
withDefaults(
  defineProps<{
    /** 0–100 when determinate; omit or null for indeterminate. */
    value?: number | null;
    label?: string;
  }>(),
  {
    value: null,
    label: undefined,
  },
);

function clamped(value: number): number {
  return Math.min(100, Math.max(0, value));
}
</script>

<template>
  <div class="flex flex-col gap-1">
    <div
      v-if="label"
      class="text-xs text-ink-muted"
    >
      {{ label }}
    </div>
    <div
      class="progress-track h-2 overflow-hidden rounded bg-surface-sunken"
      role="progressbar"
      :aria-valuenow="value === null || value === undefined ? undefined : clamped(value)"
      aria-valuemin="0"
      aria-valuemax="100"
    >
      <div
        v-if="value === null || value === undefined"
        class="progress-indeterminate h-full w-1/3 bg-accent"
      />
      <div
        v-else
        class="h-full bg-accent duration-150"
        :style="{ width: `${clamped(value)}%` }"
      />
    </div>
  </div>
</template>

<style lang="scss" scoped>
.progress-indeterminate {
  animation: progress-slide 1.2s ease-in-out infinite;
}
</style>
