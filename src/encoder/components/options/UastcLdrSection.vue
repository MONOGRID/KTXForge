<script setup lang="ts">
import { Sparkles } from "@lucide/vue";
import { computed } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import { anyEnabled } from "./optionControls";

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

const ldr = computed(() => store.options.uastcLdr);

const notes = computed(() => {
  const set = new Set<OptionKey>(UASTC_KEYS);
  return store.report.warnings.filter(
    (entry) => entry.key !== null && set.has(entry.key),
  );
});

function optionalFromValue(value: string | number): number | null {
  const text = String(value).trim();
  if (text === "") {
    return null;
  }
  const parsed = Number(text);
  return Number.isFinite(parsed) ? parsed : null;
}

function optionalIntFromValue(value: string | number): number | null {
  const parsed = optionalFromValue(value);
  return parsed === null ? null : Math.trunc(parsed);
}

function textOrEmpty(value: number | null): string {
  return value === null ? "" : String(value);
}
</script>

<template>
  <Panel
    v-if="anyEnabled(UASTC_KEYS)"
    title="UASTC LDR"
    :icon="Sparkles"
  >
    <OptionField
      v-slot="{ controlId }"
      label="UASTC quality"
      option-key="uastc-quality"
      :state="fieldState('uastc-quality')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="0"
        max="4"
        step="1"
        :model-value="textOrEmpty(ldr.uastcQuality)"
        @update:model-value="store.options.uastcLdr.uastcQuality = optionalIntFromValue($event)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="UASTC RDO"
      option-key="uastc-rdo"
      :state="fieldState('uastc-rdo')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="ldr.uastcRdo"
        @update:checked="store.options.uastcLdr.uastcRdo = $event"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO lambda"
      option-key="uastc-rdo-l"
      :state="fieldState('uastc-rdo-l')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="0.001"
        max="10"
        step="0.001"
        :model-value="textOrEmpty(ldr.uastcRdoL)"
        @update:model-value="store.options.uastcLdr.uastcRdoL = optionalFromValue($event)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO dictionary"
      option-key="uastc-rdo-d"
      :state="fieldState('uastc-rdo-d')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="64"
        max="65536"
        step="1"
        :model-value="textOrEmpty(ldr.uastcRdoD)"
        @update:model-value="store.options.uastcLdr.uastcRdoD = optionalIntFromValue($event)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO smooth-block scale"
      option-key="uastc-rdo-b"
      :state="fieldState('uastc-rdo-b')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="1"
        max="300"
        step="1"
        :model-value="textOrEmpty(ldr.uastcRdoB)"
        @update:model-value="store.options.uastcLdr.uastcRdoB = optionalFromValue($event)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO smooth-block stddev"
      option-key="uastc-rdo-s"
      :state="fieldState('uastc-rdo-s')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="0.01"
        max="65536"
        step="0.01"
        :model-value="textOrEmpty(ldr.uastcRdoS)"
        @update:model-value="store.options.uastcLdr.uastcRdoS = optionalFromValue($event)"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO: do not favor simpler modes"
      option-key="uastc-rdo-f"
      :state="fieldState('uastc-rdo-f')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="ldr.uastcRdoF"
        @update:checked="store.options.uastcLdr.uastcRdoF = $event"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="RDO: disable multithreading"
      option-key="uastc-rdo-m"
      :state="fieldState('uastc-rdo-m')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="ldr.uastcRdoM"
        @update:checked="store.options.uastcLdr.uastcRdoM = $event"
      />
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
