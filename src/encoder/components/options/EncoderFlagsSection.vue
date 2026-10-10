<script setup lang="ts">
import { computed } from "vue";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

const boxClass =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

const encoder = computed(() => store.options.encoder);

const flagWarnings = computed(() => {
  const keys = new Set(["threads", "no-sse", "compare-ssim", "compare-psnr"]);
  return store.report.warnings.filter(
    (w) => w.key !== null && keys.has(w.key),
  );
});

function writeThreads(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  const text = el.value.trim();
  if (text === "") {
    store.options.encoder.threads = null;
    return;
  }
  const n = Number(text);
  store.options.encoder.threads = Number.isFinite(n) ? Math.trunc(n) : null;
}

function writeNoSse(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.encoder.noSse = el.checked;
  }
}

function writeCompareSsim(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.encoder.compareSsim = el.checked;
  }
}

function writeComparePsnr(event: Event): void {
  const el = event.target;
  if (el instanceof HTMLInputElement) {
    store.options.encoder.comparePsnr = el.checked;
  }
}
</script>

<template>
  <Panel title="Encoder flags">
    <OptionField
      label="Threads"
      option-key="threads"
      :state="fieldState('threads')"
    >
      <input
        :class="boxClass"
        type="number"
        min="1"
        step="1"
        :value="encoder.threads === null ? '' : String(encoder.threads)"
        placeholder="tool default"
        @input="writeThreads"
      >
    </OptionField>
    <OptionField
      label="No SSE"
      option-key="no-sse"
      :state="fieldState('no-sse')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="encoder.noSse"
        @change="writeNoSse"
      >
    </OptionField>
    <OptionField
      label="Compare SSIM"
      option-key="compare-ssim"
      :state="fieldState('compare-ssim')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="encoder.compareSsim"
        @change="writeCompareSsim"
      >
    </OptionField>
    <OptionField
      label="Compare PSNR"
      option-key="compare-psnr"
      :state="fieldState('compare-psnr')"
    >
      <input
        type="checkbox"
        class="mt-1"
        :checked="encoder.comparePsnr"
        @change="writeComparePsnr"
      >
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
