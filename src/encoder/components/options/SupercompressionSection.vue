<script setup lang="ts">
import { Archive } from "@lucide/vue";
import { computed, useId } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import type { Supercompression } from "../../../bindings/Supercompression";
import { Input } from "@/components/ui/input";
import EnumSelect from "../../../shared/components/EnumSelect.vue";
import HelpTooltip from "../../../shared/components/HelpTooltip.vue";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import { anyEnabled } from "./optionControls";

type Kind = Supercompression["kind"];

const KEYS = ["zstd", "zlib"] as const satisfies readonly OptionKey[];

const schemeLabels = {
  none: "None",
  zstd: "Zstd",
  zlib: "Zlib",
} satisfies Record<Kind, string>;

const FALLBACK_ZSTD = 3;
const FALLBACK_ZLIB = 6;

const schemeId = useId();

const scheme = computed(() => store.options.supercompression.kind);

const zstdEnabled = computed(() => fieldState("zstd").enabled);
const zlibEnabled = computed(() => fieldState("zlib").enabled);

const schemes = computed(() => {
  const helps = {
    none: "No supercompression after encode.",
    zstd: fieldState("zstd").help,
    zlib: fieldState("zlib").help,
  } satisfies Record<Kind, string>;
  const all = (["none", "zstd", "zlib"] as const).map((value) => ({
    value,
    label: schemeLabels[value],
    help: helps[value],
  }));
  return all.filter((entry) => {
    if (entry.value === "zstd") {
      return zstdEnabled.value;
    }
    if (entry.value === "zlib") {
      return zlibEnabled.value;
    }
    return true;
  });
});

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

const panelVisible = computed(
  () => anyEnabled(KEYS) || store.options.supercompression.kind === "none",
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
  <Panel
    v-if="panelVisible"
    title="Supercompression"
    :icon="Archive"
  >
    <div class="field-row">
      <HelpTooltip
        text="Optional Zstd or Zlib supercompression after encode. Not valid with BasisLZ. Zstd levels above 20 use much more memory."
        class="label"
      >
        <label
          class="min-w-0 cursor-pointer truncate"
          :for="schemeId"
          title="Scheme"
        >Scheme</label>
      </HelpTooltip>
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
      v-if="scheme === 'zstd'"
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
        @update:model-value="changeZstd"
      />
    </OptionField>
    <OptionField
      v-if="scheme === 'zlib'"
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
