<script setup lang="ts">
import { computed, useId, watch } from "vue";
import type { AnimationOptions } from "../../bindings/AnimationOptions";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import Panel from "../../shared/components/Panel.vue";
import { store } from "../store";

const DEFAULT_ANIMATION = {
  duration: 1,
  timescale: 10,
  loopCount: 0,
} as const satisfies AnimationOptions;

const enableId = useId();
const durationId = useId();
const timescaleId = useId();
const loopCountId = useId();

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

function writeU32(
  field: keyof AnimationOptions,
  value: string | number,
): void {
  const current = store.animation;
  if (current === null) {
    return;
  }
  const text = String(value).trim();
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
      <div class="field-row">
        <label
          class="label"
          :for="enableId"
          title="Enable"
        >Enable</label>
        <div class="min-w-0 flex-1">
          <Switch
            :id="enableId"
            class="mt-0.5"
            :checked="enabled"
            :disabled="!canEnable"
            @update:checked="setEnabled"
          />
          <p
            v-if="!canEnable"
            class="hint"
          >
            Add at least two sources to encode an animation.
          </p>
        </div>
      </div>

      <div class="field-row">
        <label
          class="label"
          :for="durationId"
          title="Duration"
        >Duration</label>
        <Input
          :id="durationId"
          type="number"
          min="0"
          step="1"
          class="h-8 min-w-0 font-mono"
          :disabled="!enabled"
          :model-value="anim.duration"
          @update:model-value="writeU32('duration', $event)"
        />
      </div>

      <div class="field-row">
        <label
          class="label"
          :for="timescaleId"
          title="Timescale"
        >Timescale</label>
        <Input
          :id="timescaleId"
          type="number"
          min="1"
          step="1"
          class="h-8 min-w-0 font-mono"
          :disabled="!enabled"
          :model-value="anim.timescale"
          @update:model-value="writeU32('timescale', $event)"
        />
      </div>

      <div class="field-row">
        <label
          class="label"
          :for="loopCountId"
          title="Loop count"
        >Loop count</label>
        <Input
          :id="loopCountId"
          type="number"
          min="0"
          step="1"
          class="h-8 min-w-0 font-mono"
          :disabled="!enabled"
          :model-value="anim.loopCount"
          @update:model-value="writeU32('loopCount', $event)"
        />
      </div>

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
