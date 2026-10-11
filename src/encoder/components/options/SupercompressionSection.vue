<script setup lang="ts">
import { computed, useId } from "vue";
import type { Supercompression } from "../../../bindings/Supercompression";
import { Input } from "@/components/ui/input";
import EnumSelect from "../../../shared/components/EnumSelect.vue";
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

const FALLBACK_ZSTD = 3;
const FALLBACK_ZLIB = 6;

const schemeId = useId();

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

function clampInt(raw: string | number, min: number, max: number, fallback: number): number {
  const n = Number(raw);
  if (!Number.isFinite(n)) {
    return fallback;
  }
  return Math.min(max, Math.max(min, Math.trunc(n)));
}

function changeScheme(value: string | null): void {
  if (value === null || !(value in schemeLabels)) {
    return;
  }
  if (value === "none") {
    store.options.supercompression = { kind: "none" };
  } else if (value === "zstd") {
    store.options.supercompression = { kind: "zstd", level: zstdValue.value };
  } else if (value === "zlib") {
    store.options.supercompression = { kind: "zlib", level: zlibValue.value };
  }
}

function changeZstd(value: string | number): void {
  store.options.supercompression = {
    kind: "zstd",
    level: clampInt(value, 1, 22, FALLBACK_ZSTD),
  };
}

function changeZlib(value: string | number): void {
  store.options.supercompression = {
    kind: "zlib",
    level: clampInt(value, 1, 9, FALLBACK_ZLIB),
  };
}
</script>

<template>
  <Panel title="Supercompression">
    <div class="field-row">
      <label
        class="label"
        :for="schemeId"
        title="Scheme"
      >Scheme</label>
      <div class="min-w-0 flex-1">
        <EnumSelect
          :id="schemeId"
          :model-value="scheme"
          :options="schemes"
          :empty-label="false"
          @update:model-value="changeScheme"
        />
      </div>
    </div>
    <OptionField
      v-slot="{ controlId }"
      label="Zstd level"
      option-key="zstd"
      :state="fieldState('zstd')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="1"
        max="22"
        step="1"
        :model-value="zstdValue"
        :disabled="scheme === 'zlib'"
        @update:model-value="changeZstd"
      />
    </OptionField>
    <OptionField
      v-slot="{ controlId }"
      label="Zlib level"
      option-key="zlib"
      :state="fieldState('zlib')"
    >
      <Input
        :id="controlId"
        class="h-8 font-mono"
        type="number"
        min="1"
        max="9"
        step="1"
        :model-value="zlibValue"
        :disabled="scheme === 'zstd'"
        @update:model-value="changeZlib"
      />
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
