<script setup lang="ts">
import { computed, watch } from "vue";
import type { AnimationOptions } from "../../bindings/AnimationOptions";
import Panel from "../../shared/components/Panel.vue";
import { store } from "../store";

const DEFAULT_ANIMATION = {
  duration: 1,
  timescale: 10,
  loopCount: 0,
} as const satisfies AnimationOptions;

const canEnable = computed((): boolean => store.sources.length >= 2);

const enabled = computed((): boolean => store.animation !== null);

const anim = computed((): AnimationOptions => {
  return store.animation ?? { ...DEFAULT_ANIMATION };
});

const frameLengthSec = computed((): number => {
  const { duration, timescale } = anim.value;
  if (timescale === 0) {
    return 0;
  }
  return duration / timescale;
});

const sequenceLengthSec = computed(
  (): number => frameLengthSec.value * store.sources.length,
);

watch(canEnable, (ok) => {
  if (!ok && store.animation !== null) {
    store.animation = null;
  }
});

function formatSeconds(value: number): string {
  if (!Number.isFinite(value)) {
    return "—";
  }
  return `${value.toFixed(3)} s`;
}

function setEnabled(next: boolean): void {
  if (!canEnable.value) {
    store.animation = null;
    return;
  }
  store.animation = next ? { ...DEFAULT_ANIMATION } : null;
}

function onEnableChange(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    setEnabled(el.checked);
  }
}

function writeU32(
  field: keyof AnimationOptions,
  event: Event,
): void {
  const current = store.animation;
  if (current === null) {
    return;
  }
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  const text = el.value.trim();
  if (text === "") {
    return;
  }
  const n = Number(text);
  if (!Number.isFinite(n) || n < 0) {
    return;
  }
  current[field] = Math.trunc(n);
}
</script>

<template>
  <Panel title="Animation">
    <div class="flex flex-col gap-2">
      <label class="field-row">
        <span class="label">Enable</span>
        <div class="min-w-0 flex-1">
          <input
            type="checkbox"
            class="mt-1"
            :checked="enabled"
            :disabled="!canEnable"
            @change="onEnableChange"
          >
          <p
            v-if="!canEnable"
            class="hint"
          >
            Add at least two sources to encode an animation.
          </p>
        </div>
      </label>

      <label class="field-row">
        <span class="label">Duration</span>
        <input
          type="number"
          min="0"
          step="1"
          class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
          :disabled="!enabled"
          :value="anim.duration"
          @change="writeU32('duration', $event)"
        >
      </label>

      <label class="field-row">
        <span class="label">Timescale</span>
        <input
          type="number"
          min="1"
          step="1"
          class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
          :disabled="!enabled"
          :value="anim.timescale"
          @change="writeU32('timescale', $event)"
        >
      </label>

      <label class="field-row">
        <span class="label">Loop count</span>
        <input
          type="number"
          min="0"
          step="1"
          class="w-full rounded border border-line bg-surface px-2 py-1 text-sm font-mono"
          :disabled="!enabled"
          :value="anim.loopCount"
          @change="writeU32('loopCount', $event)"
        >
      </label>

      <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs font-mono text-ink-muted">
        <span>Frame {{ formatSeconds(frameLengthSec) }}</span>
        <span>Sequence {{ formatSeconds(sequenceLengthSec) }}</span>
        <span>
          Loop
          {{ anim.loopCount === 0 ? "∞" : String(anim.loopCount) }}
        </span>
      </div>
    </div>
  </Panel>
</template>
