<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4
import { computed } from "vue";
import type { AstcQuality } from "../../../bindings/AstcQuality";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

const qualityLabels = {
  fastest: "Fastest (0)",
  fast: "Fast (10)",
  medium: "Medium (60)",
  thorough: "Thorough (98)",
  exhaustive: "Exhaustive (100)",
} satisfies Record<AstcQuality, string>;

const qualityChoices = (
  ["fastest", "fast", "medium", "thorough", "exhaustive"] as const
).map((value) => ({ value, label: qualityLabels[value] }));

const selectClass =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

const qualityState = computed(() => fieldState("astc-quality"));
const perceptualState = computed(() => fieldState("astc-perceptual"));

const astcWarnings = computed(() =>
  store.report.warnings.filter(
    (w) => w.key === "astc-quality" || w.key === "astc-perceptual",
  ),
);

function isQuality(value: string): value is AstcQuality {
  return value in qualityLabels;
}

function pickQuality(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement)) {
    return;
  }
  if (el.value === "") {
    store.options.astc.astcQuality = null;
    return;
  }
  if (isQuality(el.value)) {
    store.options.astc.astcQuality = el.value;
  }
}

function togglePerceptual(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.astc.astcPerceptual = el.checked;
  }
}
</script>

<template>
  <Panel title="ASTC">
    <OptionField
      label="ASTC quality"
      option-key="astc-quality"
      :state="qualityState"
    >
      <select
        :class="selectClass"
        :value="store.options.astc.astcQuality ?? ''"
        @change="pickQuality"
      >
        <option value="">
          Default (medium)
        </option>
        <option
          v-for="choice in qualityChoices"
          :key="choice.value"
          :value="choice.value"
        >
          {{ choice.label }}
        </option>
      </select>
    </OptionField>
    <OptionField
      label="ASTC perceptual"
      option-key="astc-perceptual"
      :state="perceptualState"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="store.options.astc.astcPerceptual"
        @change="togglePerceptual"
      >
    </OptionField>
    <template
      v-for="(warning, idx) in astcWarnings"
      :key="`astc-warn-${String(idx)}`"
    >
      <p
        class="hint"
        :class="{ 'text-danger': warning.level === 'warn' }"
      >
        {{ warning.message }}
      </p>
    </template>
  </Panel>
</template>
