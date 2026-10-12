<script setup lang="ts">
import { Binary } from "@lucide/vue";
import { computed, useId } from "vue";
import EnumSelect from "../../../shared/components/EnumSelect.vue";
import HelpTooltip from "../../../shared/components/HelpTooltip.vue";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";
import {
  astcBlockLabels,
  astcBlockValues,
  astcColorLabels,
  astcFormat,
  encodeLabels,
  encodeValues,
  isAstcBlock,
  isAstcColor,
  isAstcFormat,
  isEncodeCodec,
  isFormatMode,
  isUncompressedFormat,
  modeHelps,
  modeLabels,
  modeValues,
  parseAstc,
  uncompressedLabels,
  type AstcBlock,
  type AstcColor,
  type FormatMode,
} from "./formatCatalog";
import {
  choiceHelp,
  enumEntriesWithHelp,
  isChoiceAllowed,
} from "./optionControls";

const uncompressedOptions = computed(() =>
  enumEntriesWithHelp("format", uncompressedLabels),
);

const encodeOptions = computed(() =>
  enumEntriesWithHelp(
    "encode",
    Object.fromEntries(encodeValues.map((value) => [value, encodeLabels[value]])),
  ),
);

/** ASTC formats Rust allows for this block / color pair. */
function astcAllowed(block: AstcBlock, color: AstcColor): boolean {
  return isChoiceAllowed("format", astcFormat(block, color));
}

const astcBlockOptions = computed(() =>
  astcBlockValues
    .filter((value) => astcAllowed(value, "UNORM") || astcAllowed(value, "SRGB"))
    .map((value) => ({
      value,
      label: astcBlockLabels[value],
      help:
        choiceHelp("format", `ASTC_${value}_UNORM_BLOCK`) ??
        `ASTC ${astcBlockLabels[value]} block size.`,
    })),
);

const astcColorHelps = {
  UNORM: "Linear UNORM ASTC (no sRGB decode in the GPU sampler).",
  SRGB: "sRGB ASTC. Use for color textures viewed in sRGB.",
} satisfies Record<AstcColor, string>;

const astcColorOptions = computed(() =>
  (["UNORM", "SRGB"] as const)
    .filter((color) => astcAllowed(astcParts.value.block, color))
    .map((value) => ({
      value,
      label: astcColorLabels[value],
      help: astcColorHelps[value],
    })),
);

/** Modes with at least one choice Rust allows for the current sources. */
const allowedModes = computed((): readonly FormatMode[] =>
  modeValues.filter((value) => {
    if (value === "astc") {
      return astcBlockOptions.value.length > 0;
    }
    if (value === "codec") {
      return encodeOptions.value.length > 0;
    }
    return uncompressedOptions.value.length > 0;
  }),
);

const modeOptions = computed(() =>
  allowedModes.value.map((value) => ({
    value,
    label: modeLabels[value],
    help: modeHelps[value],
  })),
);

const groupId = useId();

const mode = computed<FormatMode>(() => {
  if (store.options.encode !== null) {
    return "codec";
  }
  if (isAstcFormat(store.options.format)) {
    return "astc";
  }
  return "uncompressed";
});

const astcParts = computed(() => parseAstc(store.options.format));

const formatWarnings = computed(() =>
  store.report.warnings.filter(
    (w) => w.key === "format" || w.key === "encode",
  ),
);

function ensureUncompressedFormat(): void {
  if (!isUncompressedFormat(store.options.format)) {
    store.options.format = store.report.defaultFormat;
  }
}

function setUncompressedMode(): void {
  ensureUncompressedFormat();
  store.options.encode = null;
}

function setAstcMode(): void {
  const parts = parseAstc(store.options.format);
  store.options.format = astcFormat(parts.block, parts.color);
  store.options.encode = null;
}

function setCodecMode(): void {
  ensureUncompressedFormat();
  const first = encodeOptions.value.map((option) => option.value).find(isEncodeCodec);
  if (store.options.encode === null && first !== undefined) {
    store.options.encode = first;
  }
}

function setMode(next: FormatMode): void {
  if (!allowedModes.value.includes(next)) {
    return;
  }
  if (next === "uncompressed") {
    setUncompressedMode();
    return;
  }
  if (next === "astc") {
    setAstcMode();
    return;
  }
  setCodecMode();
}

function onModeChange(value: string | null): void {
  if (value !== null && isFormatMode(value)) {
    setMode(value);
  }
}

function onUncompressedFormat(value: string | null): void {
  if (value !== null && isUncompressedFormat(value)) {
    store.options.format = value;
    if (mode.value !== "codec") {
      store.options.encode = null;
    }
  }
}

function onAstcBlock(value: string | null): void {
  if (value !== null && isAstcBlock(value)) {
    store.options.format = astcFormat(value, astcParts.value.color);
    store.options.encode = null;
  }
}

function onAstcColor(value: string | null): void {
  if (value !== null && isAstcColor(value)) {
    store.options.format = astcFormat(astcParts.value.block, value);
    store.options.encode = null;
  }
}

function onEncode(value: string | null): void {
  if (value === null) {
    store.options.encode = null;
    return;
  }
  if (isEncodeCodec(value) && isChoiceAllowed("encode", value)) {
    ensureUncompressedFormat();
    store.options.encode = value;
  }
}

const uncompressedGroups = computed(() => [
  {
    label:
      mode.value === "codec" ? "Uncompressed (with --encode)" : "Uncompressed",
    options: uncompressedOptions.value,
  },
]);

const astcBlockGroups = computed(() => [
  { label: "ASTC block size", options: astcBlockOptions.value },
]);

const astcColorGroups = computed(() => [
  { label: "sRGB / UNORM", options: astcColorOptions.value },
]);
</script>

<template>
  <Panel
    title="Format"
    :icon="Binary"
  >
    <div class="field-row">
      <HelpTooltip
        text="Uncompressed Vulkan format, ASTC block format, or uncompressed plus BasisLZ/UASTC encode."
        class="label"
      >
        <label
          class="min-w-0 cursor-pointer truncate"
          :for="groupId"
          title="Group"
        >Group</label>
      </HelpTooltip>
      <div class="min-w-0 flex-1">
        <EnumSelect
          :id="groupId"
          :model-value="mode"
          :options="modeOptions"
          :empty-label="false"
          @update:model-value="onModeChange"
        />
      </div>
    </div>

    <OptionField
      label="Format"
      option-key="format"
      :state="fieldState('format')"
    >
      <EnumSelect
        v-if="mode === 'uncompressed' || mode === 'codec'"
        :model-value="store.options.format"
        :groups="uncompressedGroups"
        :empty-label="false"
        @update:model-value="onUncompressedFormat"
      />
      <div
        v-else
        class="flex flex-col gap-2"
      >
        <EnumSelect
          :model-value="astcParts.block"
          :groups="astcBlockGroups"
          :empty-label="false"
          aria-label="ASTC block size"
          @update:model-value="onAstcBlock"
        />
        <EnumSelect
          :model-value="astcParts.color"
          :groups="astcColorGroups"
          :empty-label="false"
          aria-label="ASTC color space"
          @update:model-value="onAstcColor"
        />
      </div>
    </OptionField>

    <OptionField
      v-if="mode !== 'astc'"
      label="Encode"
      option-key="encode"
      :state="fieldState('encode')"
    >
      <EnumSelect
        :model-value="store.options.encode"
        :options="encodeOptions"
        empty-label="None"
        empty-help="Leave uncompressed; no BasisLZ or UASTC encode step."
        @update:model-value="onEncode"
      />
    </OptionField>

    <div
      v-if="formatWarnings.length > 0"
      class="mt-1 space-y-0.5"
    >
      <span
        v-for="(warning, index) in formatWarnings"
        :key="`fmt-${String(index)}`"
        class="hint block"
        :class="{ 'text-danger': warning.level === 'warn' }"
      >{{ warning.message }}</span>
    </div>
  </Panel>
</template>
