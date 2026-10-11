<script setup lang="ts">
import { computed } from "vue";
import type { AstcQuality } from "../../../bindings/AstcQuality";
import type { OptionKey } from "../../../bindings/OptionKey";
import { Switch } from "@/components/ui/switch";
import EnumSelect from "../../../shared/components/EnumSelect.vue";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import { anyEnabled } from "./optionControls";

const KEYS = ["astc-quality", "astc-perceptual"] as const satisfies readonly OptionKey[];

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

function pickQuality(value: string | null): void {
  if (value === null) {
    store.options.astc.astcQuality = null;
    return;
  }
  if (isQuality(value)) {
    store.options.astc.astcQuality = value;
  }
}
</script>

<template>
  <Panel
    v-if="anyEnabled(KEYS)"
    title="ASTC"
  >
    <OptionField
      label="ASTC quality"
      option-key="astc-quality"
      :state="qualityState"
    >
      <EnumSelect
        :model-value="store.options.astc.astcQuality"
        :options="qualityChoices"
        :empty-label="false"
        @update:model-value="pickQuality"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="ASTC perceptual"
      option-key="astc-perceptual"
      :state="perceptualState"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="store.options.astc.astcPerceptual"
        @update:checked="store.options.astc.astcPerceptual = $event"
      />
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
