<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4
import { computed } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

const UASTC_KEYS = [
  "uastc-quality",
  "uastc-rdo",
  "uastc-rdo-l",
  "uastc-rdo-d",
  "uastc-rdo-b",
  "uastc-rdo-s",
  "uastc-rdo-f",
  "uastc-rdo-m",
] as const satisfies readonly OptionKey[];

const fieldClass =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

const ldr = computed(() => store.options.uastcLdr);

const notes = computed(() => {
  const set = new Set<OptionKey>(UASTC_KEYS);
  return store.report.warnings.filter(
    (entry) => entry.key !== null && set.has(entry.key),
  );
});

function optionalFromInput(event: Event): number | null {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return null;
  }
  const text = el.value.trim();
  if (text === "") {
    return null;
  }
  const parsed = Number(text);
  return Number.isFinite(parsed) ? parsed : null;
}

function optionalIntFromInput(event: Event): number | null {
  const value = optionalFromInput(event);
  return value === null ? null : Math.trunc(value);
}

function checkedFromInput(event: Event): boolean {
  const el = event.target;
  return el instanceof HTMLInputElement ? el.checked : false;
}

function textOrEmpty(value: number | null): string {
  return value === null ? "" : String(value);
}
</script>

<template>
  <Panel title="UASTC LDR">
    <OptionField
      label="UASTC quality"
      option-key="uastc-quality"
      :state="fieldState('uastc-quality')"
    >
      <input
        :class="fieldClass"
        type="number"
        min="0"
        max="4"
        step="1"
        :value="textOrEmpty(ldr.uastcQuality)"
        placeholder="tool default"
        @input="store.options.uastcLdr.uastcQuality = optionalIntFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="UASTC RDO"
      option-key="uastc-rdo"
      :state="fieldState('uastc-rdo')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="ldr.uastcRdo"
        @change="store.options.uastcLdr.uastcRdo = checkedFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO lambda"
      option-key="uastc-rdo-l"
      :state="fieldState('uastc-rdo-l')"
    >
      <input
        :class="fieldClass"
        type="number"
        min="0.001"
        max="10"
        step="0.001"
        :value="textOrEmpty(ldr.uastcRdoL)"
        placeholder="tool default"
        @input="store.options.uastcLdr.uastcRdoL = optionalFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO dictionary"
      option-key="uastc-rdo-d"
      :state="fieldState('uastc-rdo-d')"
    >
      <input
        :class="fieldClass"
        type="number"
        min="64"
        max="65536"
        step="1"
        :value="textOrEmpty(ldr.uastcRdoD)"
        placeholder="tool default"
        @input="store.options.uastcLdr.uastcRdoD = optionalIntFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO smooth-block scale"
      option-key="uastc-rdo-b"
      :state="fieldState('uastc-rdo-b')"
    >
      <input
        :class="fieldClass"
        type="number"
        min="1"
        max="300"
        step="1"
        :value="textOrEmpty(ldr.uastcRdoB)"
        placeholder="tool default"
        @input="store.options.uastcLdr.uastcRdoB = optionalFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO smooth-block stddev"
      option-key="uastc-rdo-s"
      :state="fieldState('uastc-rdo-s')"
    >
      <input
        :class="fieldClass"
        type="number"
        min="0.01"
        max="65536"
        step="0.01"
        :value="textOrEmpty(ldr.uastcRdoS)"
        placeholder="tool default"
        @input="store.options.uastcLdr.uastcRdoS = optionalFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO: do not favor simpler modes"
      option-key="uastc-rdo-f"
      :state="fieldState('uastc-rdo-f')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="ldr.uastcRdoF"
        @change="store.options.uastcLdr.uastcRdoF = checkedFromInput($event)"
      >
    </OptionField>
    <OptionField
      label="RDO: disable multithreading"
      option-key="uastc-rdo-m"
      :state="fieldState('uastc-rdo-m')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="ldr.uastcRdoM"
        @change="store.options.uastcLdr.uastcRdoM = checkedFromInput($event)"
      >
    </OptionField>
    <div
      v-for="(note, index) in notes"
      :key="`uastc-${note.message}-${String(index)}`"
      class="hint"
      :class="note.level === 'warn' ? 'text-danger' : undefined"
    >
      {{ note.message }}
    </div>
  </Panel>
</template>
