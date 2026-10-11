<script setup lang="ts">
import { computed } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import { anyEnabled } from "./optionControls";

const KEYS = [
  "threads",
  "no-sse",
  "compare-ssim",
  "compare-psnr",
] as const satisfies readonly OptionKey[];

const encoder = computed(() => store.options.encoder);

const flagWarnings = computed(() => {
  const keys = new Set<string>(KEYS);
  return store.report.warnings.filter(
    (w) => w.key !== null && keys.has(w.key),
  );
});

function writeThreads(value: string | number): void {
  const text = String(value).trim();
  if (text === "") {
    store.options.encoder.threads = null;
    return;
  }
  const n = Number(text);
  store.options.encoder.threads = Number.isFinite(n) ? Math.trunc(n) : null;
}
</script>

<template>
  <Panel
    v-if="anyEnabled(KEYS)"
    title="Encoder flags"
  >
    <OptionField
      v-slot="{ controlId }"
      label="Threads"
      option-key="threads"
      :state="fieldState('threads')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="1"
        step="1"
        :model-value="encoder.threads === null ? '' : String(encoder.threads)"
        @update:model-value="writeThreads"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="No SSE"
      option-key="no-sse"
      :state="fieldState('no-sse')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="encoder.noSse"
        @update:checked="store.options.encoder.noSse = $event"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Compare SSIM"
      option-key="compare-ssim"
      :state="fieldState('compare-ssim')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="encoder.compareSsim"
        @update:checked="store.options.encoder.compareSsim = $event"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Compare PSNR"
      option-key="compare-psnr"
      :state="fieldState('compare-psnr')"
    >
      <Switch
        :id="controlId"
        class="mt-0.5"
        :checked="encoder.comparePsnr"
        @update:checked="store.options.encoder.comparePsnr = $event"
      />
    </OptionField>
    <aside
      v-if="flagWarnings.length > 0"
      class="mt-1 flex flex-col gap-0.5"
    >
      <span
        v-for="(w, n) in flagWarnings"
        :key="`enc-${String(n)}`"
        class="hint"
        :class="w.level === 'warn' ? 'text-danger' : 'text-ink-faint'"
      >{{ w.message }}</span>
    </aside>
  </Panel>
</template>
