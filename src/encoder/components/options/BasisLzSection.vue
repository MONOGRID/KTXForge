<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4
import { computed } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

const KEYS = [
  "qlevel",
  "clevel",
  "max-endpoints",
  "max-selectors",
  "endpoint-rdo-threshold",
  "selector-rdo-threshold",
  "no-endpoint-rdo",
  "no-selector-rdo",
] as const satisfies readonly OptionKey[];

const inputClass =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

const basis = computed(() => store.options.basisLz);

const sectionWarnings = computed(() =>
  store.report.warnings.filter(
    (w) => w.key !== null && KEYS.some((k) => k === w.key),
  ),
);

function readOptional(raw: string, integer: boolean): number | null {
  if (raw.trim() === "") {
    return null;
  }
  const n = Number(raw);
  if (!Number.isFinite(n)) {
    return null;
  }
  return integer ? Math.trunc(n) : n;
}

function onQlevel(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.qlevel = readOptional(el.value, true);
  }
}

function onClevel(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.clevel = readOptional(el.value, true);
  }
}

function onMaxEndpoints(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.maxEndpoints = readOptional(el.value, true);
  }
}

function onMaxSelectors(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.maxSelectors = readOptional(el.value, true);
  }
}

function onEndpointRdo(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.endpointRdoThreshold = readOptional(el.value, false);
  }
}

function onSelectorRdo(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.selectorRdoThreshold = readOptional(el.value, false);
  }
}

function onNoEndpointRdo(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.noEndpointRdo = el.checked;
  }
}

function onNoSelectorRdo(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.basisLz.noSelectorRdo = el.checked;
  }
}

function shown(value: number | null): string {
  return value === null ? "" : String(value);
}
</script>

<template>
  <Panel title="BasisLZ">
    <OptionField
      label="Quality level"
      option-key="qlevel"
      :state="fieldState('qlevel')"
    >
      <input
        :class="inputClass"
        type="number"
        min="1"
        max="255"
        step="1"
        :value="shown(basis.qlevel)"
        placeholder="tool default"
        @input="onQlevel"
      >
    </OptionField>
    <OptionField
      label="Compression level"
      option-key="clevel"
      :state="fieldState('clevel')"
    >
      <input
        :class="inputClass"
        type="number"
        min="0"
        max="6"
        step="1"
        :value="shown(basis.clevel)"
        placeholder="tool default"
        @input="onClevel"
      >
    </OptionField>
    <OptionField
      label="Max endpoints"
      option-key="max-endpoints"
      :state="fieldState('max-endpoints')"
    >
      <input
        :class="inputClass"
        type="number"
        min="1"
        max="16128"
        step="1"
        :value="shown(basis.maxEndpoints)"
        placeholder="tool default"
        @input="onMaxEndpoints"
      >
    </OptionField>
    <OptionField
      label="Max selectors"
      option-key="max-selectors"
      :state="fieldState('max-selectors')"
    >
      <input
        :class="inputClass"
        type="number"
        min="1"
        max="16128"
        step="1"
        :value="shown(basis.maxSelectors)"
        placeholder="tool default"
        @input="onMaxSelectors"
      >
    </OptionField>
    <OptionField
      label="Endpoint RDO threshold"
      option-key="endpoint-rdo-threshold"
      :state="fieldState('endpoint-rdo-threshold')"
    >
      <input
        :class="inputClass"
        type="number"
        min="1"
        max="3"
        step="0.05"
        :value="shown(basis.endpointRdoThreshold)"
        placeholder="tool default"
        @input="onEndpointRdo"
      >
    </OptionField>
    <OptionField
      label="Selector RDO threshold"
      option-key="selector-rdo-threshold"
      :state="fieldState('selector-rdo-threshold')"
    >
      <input
        :class="inputClass"
        type="number"
        min="1"
        max="3"
        step="0.05"
        :value="shown(basis.selectorRdoThreshold)"
        placeholder="tool default"
        @input="onSelectorRdo"
      >
    </OptionField>
    <OptionField
      label="No endpoint RDO"
      option-key="no-endpoint-rdo"
      :state="fieldState('no-endpoint-rdo')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="basis.noEndpointRdo"
        @change="onNoEndpointRdo"
      >
    </OptionField>
    <OptionField
      label="No selector RDO"
      option-key="no-selector-rdo"
      :state="fieldState('no-selector-rdo')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="basis.noSelectorRdo"
        @change="onNoSelectorRdo"
      >
    </OptionField>
    <ul
      v-if="sectionWarnings.length > 0"
      class="m-0 list-none p-0"
    >
      <li
        v-for="(item, i) in sectionWarnings"
        :key="`basis-${String(i)}`"
        class="hint"
        :class="{ 'text-danger': item.level === 'warn' }"
      >
        {{ item.message }}
      </li>
    </ul>
  </Panel>
</template>
