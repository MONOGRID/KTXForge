<script setup lang="ts">
import { computed } from "vue";
import { rangeTickValues } from "../../sliderTicks";

const props = withDefaults(
  defineProps<{
    min: number;
    max: number;
    maxTicks?: number;
    format?: (value: number) => string;
  }>(),
  {
    maxTicks: 8,
  },
);

const ticks = computed((): number[] =>
  rangeTickValues(props.min, props.max, props.maxTicks),
);

function labelFor(value: number): string {
  return props.format === undefined ? String(value) : props.format(value);
}

function leftPercent(value: number): string {
  if (props.max <= props.min) {
    return "0%";
  }
  return `${String(((value - props.min) / (props.max - props.min)) * 100)}%`;
}

function alignClass(value: number): string {
  if (value === props.min) {
    return "translate-x-0 text-left";
  }
  if (value === props.max) {
    return "-translate-x-full text-right";
  }
  return "-translate-x-1/2 text-center";
}
</script>

<template>
  <div
    class="relative mt-0.5 h-3.5 w-full"
    aria-hidden="true"
  >
    <span
      v-for="tick in ticks"
      :key="tick"
      class="absolute top-0 font-mono text-[10px] leading-none text-ink-faint tabular-nums"
      :class="alignClass(tick)"
      :style="{ left: leftPercent(tick) }"
    >{{ labelFor(tick) }}</span>
  </div>
</template>
