<script setup lang="ts">
import { Package } from "@lucide/vue";
import { computed } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import { anyEnabled } from "./optionControls";

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

const basis = computed(() => store.options.basisLz);

const sectionWarnings = computed(() =>
  store.report.warnings.filter(
    (w) => w.key !== null && KEYS.some((k) => k === w.key),
  ),
);

function readOptional(raw: string | number, integer: boolean): number | null {
  const text = String(raw).trim();
  if (text === "") {
    return null;
  }
  const n = Number(text);
  if (!Number.isFinite(n)) {
    return null;
  }
  return integer ? Math.trunc(n) : n;
}

function shown(value: number | null): string {
  return value === null ? "" : String(value);
}
</script>

<template>
  <Panel
    v-if="anyEnabled(KEYS)"
    title="BasisLZ"
    :icon="Package"
  >
    <OptionField
      v-slot="{ controlId }"
      label="Quality level"
      option-key="qlevel"
      :state="fieldState('qlevel')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="1"
        max="255"
        step="1"
        :model-value="shown(basis.qlevel)"
        @update:model-value="store.options.basisLz.qlevel = readOptional($event, true)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Compression level"
      option-key="clevel"
      :state="fieldState('clevel')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="0"
        max="6"
        step="1"
        :model-value="shown(basis.clevel)"
        @update:model-value="store.options.basisLz.clevel = readOptional($event, true)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Max endpoints"
      option-key="max-endpoints"
      :state="fieldState('max-endpoints')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="1"
        max="16128"
        step="1"
        :model-value="shown(basis.maxEndpoints)"
        @update:model-value="store.options.basisLz.maxEndpoints = readOptional($event, true)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Max selectors"
      option-key="max-selectors"
      :state="fieldState('max-selectors')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="1"
        max="16128"
        step="1"
        :model-value="shown(basis.maxSelectors)"
        @update:model-value="store.options.basisLz.maxSelectors = readOptional($event, true)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Endpoint RDO threshold"
      option-key="endpoint-rdo-threshold"
      :state="fieldState('endpoint-rdo-threshold')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="1"
        max="3"
        step="0.05"
        :model-value="shown(basis.endpointRdoThreshold)"
        @update:model-value="store.options.basisLz.endpointRdoThreshold = readOptional($event, false)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Selector RDO threshold"
      option-key="selector-rdo-threshold"
      :state="fieldState('selector-rdo-threshold')"
    >
      <Input
        :id="controlId"
        class="font-mono"
        type="number"
        min="1"
        max="3"
        step="0.05"
        :model-value="shown(basis.selectorRdoThreshold)"
        @update:model-value="store.options.basisLz.selectorRdoThreshold = readOptional($event, false)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="No endpoint RDO"
      option-key="no-endpoint-rdo"
      :state="fieldState('no-endpoint-rdo')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="basis.noEndpointRdo"
        @update:checked="store.options.basisLz.noEndpointRdo = $event"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="No selector RDO"
      option-key="no-selector-rdo"
      :state="fieldState('no-selector-rdo')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="basis.noSelectorRdo"
        @update:checked="store.options.basisLz.noSelectorRdo = $event"
      />
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
