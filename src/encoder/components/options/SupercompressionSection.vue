<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4
import { computed } from "vue";
import type { Supercompression } from "../../../bindings/Supercompression";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

type Kind = Supercompression["kind"];

const schemeLabels = {
  none: "None",
  zstd: "Zstd",
  zlib: "Zlib",
} satisfies Record<Kind, string>;

const schemes = (["none", "zstd", "zlib"] as const).map((value) => ({
  value,
  label: schemeLabels[value],
}));

const ctl =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

const FALLBACK_ZSTD = 3;
const FALLBACK_ZLIB = 6;

const scheme = computed(() => store.options.supercompression.kind);

const zstdValue = computed(() => {
  const sc = store.options.supercompression;
  return sc.kind === "zstd" ? sc.level : FALLBACK_ZSTD;
});

const zlibValue = computed(() => {
  const sc = store.options.supercompression;
  return sc.kind === "zlib" ? sc.level : FALLBACK_ZLIB;
});

const scWarnings = computed(() =>
  store.report.warnings.filter((w) => w.key === "zstd" || w.key === "zlib"),
);

function clampInt(raw: string, min: number, max: number, fallback: number): number {
  const n = Number(raw);
  if (!Number.isFinite(n)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, Math.trunc(n)));
}

function changeScheme(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLSelectElement) || !(el.value in schemeLabels)) {
    return;
  }
  const next = el.value;
  if (next === "none") {
    store.options.supercompression = { kind: "none" };
  } else if (next === "zstd") {
    store.options.supercompression = { kind: "zstd", level: zstdValue.value };
  } else if (next === "zlib") {
    store.options.supercompression = { kind: "zlib", level: zlibValue.value };
  }
}

function changeZstd(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  store.options.supercompression = {
    kind: "zstd",
    level: clampInt(el.value, 1, 22, FALLBACK_ZSTD),
  };
}

function changeZlib(event: Event): void {
  const el = event.target;
  if (!(el instanceof HTMLInputElement)) {
    return;
  }
  store.options.supercompression = {
    kind: "zlib",
    level: clampInt(el.value, 1, 9, FALLBACK_ZLIB),
  };
}
</script>

<template>
  <Panel title="Supercompression">
    <div class="field-row">
      <div class="label">
        Scheme
      </div>
      <div class="min-w-0 flex-1">
        <select
          :class="ctl"
          :value="scheme"
          @change="changeScheme"
        >
          <option
            v-for="item in schemes"
            :key="item.value"
            :value="item.value"
          >
            {{ item.label }}
          </option>
        </select>
      </div>
    </div>
    <OptionField
      label="Zstd level"
      option-key="zstd"
      :state="fieldState('zstd')"
    >
      <input
        :class="ctl"
        type="number"
        min="1"
        max="22"
        step="1"
        :value="zstdValue"
        :disabled="scheme === 'zlib'"
        @input="changeZstd"
      >
    </OptionField>
    <OptionField
      label="Zlib level"
      option-key="zlib"
      :state="fieldState('zlib')"
    >
      <input
        :class="ctl"
        type="number"
        min="1"
        max="9"
        step="1"
        :value="zlibValue"
        :disabled="scheme === 'zstd'"
        @input="changeZlib"
      >
    </OptionField>
    <p
      v-for="msg in scWarnings"
      :key="msg.message"
      class="hint"
      :data-level="msg.level"
      :class="{ 'text-danger': msg.level === 'warn' }"
    >
      {{ msg.message }}
    </p>
  </Panel>
</template>
