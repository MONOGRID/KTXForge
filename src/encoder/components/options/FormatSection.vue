<script setup lang="ts">
import { computed } from "vue";
import type { EncodeCodec } from "../../../bindings/EncodeCodec";
import type { VkFormat } from "../../../bindings/VkFormat";
import OptionField from "../../../shared/components/OptionField.vue";
import Panel from "../../../shared/components/Panel.vue";
import { fieldState, store } from "../../store";

type FormatMode = "uncompressed" | "astc" | "codec";

type AstcBlock =
  | "4x4"
  | "5x4"
  | "5x5"
  | "6x5"
  | "6x6"
  | "8x5"
  | "8x6"
  | "8x8"
  | "10x5"
  | "10x6"
  | "10x8"
  | "10x10"
  | "12x10"
  | "12x12";

type AstcColor = "UNORM" | "SRGB";

type UncompressedFormat =
  | "R8_UNORM"
  | "R8_SRGB"
  | "R8G8_UNORM"
  | "R8G8_SRGB"
  | "R8G8B8_UNORM"
  | "R8G8B8_SRGB"
  | "R8G8B8A8_UNORM"
  | "R8G8B8A8_SRGB"
  | "R16G16B16_SFLOAT"
  | "R16G16B16A16_SFLOAT";

const uncompressedLabels = {
  R8_UNORM: "R8 UNORM",
  R8_SRGB: "R8 sRGB",
  R8G8_UNORM: "RG8 UNORM",
  R8G8_SRGB: "RG8 sRGB",
  R8G8B8_UNORM: "RGB8 UNORM",
  R8G8B8_SRGB: "RGB8 sRGB",
  R8G8B8A8_UNORM: "RGBA8 UNORM",
  R8G8B8A8_SRGB: "RGBA8 sRGB",
  R16G16B16_SFLOAT: "RGB16 SFLOAT",
  R16G16B16A16_SFLOAT: "RGBA16 SFLOAT",
} satisfies Record<UncompressedFormat, string>;

const encodeLabels = {
  "basis-lz": "BasisLZ",
  uastc: "UASTC",
} satisfies Record<EncodeCodec, string>;

const astcBlockLabels = {
  "4x4": "4×4",
  "5x4": "5×4",
  "5x5": "5×5",
  "6x5": "6×5",
  "6x6": "6×6",
  "8x5": "8×5",
  "8x6": "8×6",
  "8x8": "8×8",
  "10x5": "10×5",
  "10x6": "10×6",
  "10x8": "10×8",
  "10x10": "10×10",
  "12x10": "12×10",
  "12x12": "12×12",
} satisfies Record<AstcBlock, string>;

const astcColorLabels = {
  UNORM: "UNORM",
  SRGB: "sRGB",
} satisfies Record<AstcColor, string>;

const modeLabels = {
  uncompressed: "Uncompressed",
  astc: "ASTC",
  codec: "Codec (--encode)",
} satisfies Record<FormatMode, string>;

const astcFormats = {
  "4x4_UNORM": "ASTC_4x4_UNORM_BLOCK",
  "4x4_SRGB": "ASTC_4x4_SRGB_BLOCK",
  "5x4_UNORM": "ASTC_5x4_UNORM_BLOCK",
  "5x4_SRGB": "ASTC_5x4_SRGB_BLOCK",
  "5x5_UNORM": "ASTC_5x5_UNORM_BLOCK",
  "5x5_SRGB": "ASTC_5x5_SRGB_BLOCK",
  "6x5_UNORM": "ASTC_6x5_UNORM_BLOCK",
  "6x5_SRGB": "ASTC_6x5_SRGB_BLOCK",
  "6x6_UNORM": "ASTC_6x6_UNORM_BLOCK",
  "6x6_SRGB": "ASTC_6x6_SRGB_BLOCK",
  "8x5_UNORM": "ASTC_8x5_UNORM_BLOCK",
  "8x5_SRGB": "ASTC_8x5_SRGB_BLOCK",
  "8x6_UNORM": "ASTC_8x6_UNORM_BLOCK",
  "8x6_SRGB": "ASTC_8x6_SRGB_BLOCK",
  "8x8_UNORM": "ASTC_8x8_UNORM_BLOCK",
  "8x8_SRGB": "ASTC_8x8_SRGB_BLOCK",
  "10x5_UNORM": "ASTC_10x5_UNORM_BLOCK",
  "10x5_SRGB": "ASTC_10x5_SRGB_BLOCK",
  "10x6_UNORM": "ASTC_10x6_UNORM_BLOCK",
  "10x6_SRGB": "ASTC_10x6_SRGB_BLOCK",
  "10x8_UNORM": "ASTC_10x8_UNORM_BLOCK",
  "10x8_SRGB": "ASTC_10x8_SRGB_BLOCK",
  "10x10_UNORM": "ASTC_10x10_UNORM_BLOCK",
  "10x10_SRGB": "ASTC_10x10_SRGB_BLOCK",
  "12x10_UNORM": "ASTC_12x10_UNORM_BLOCK",
  "12x10_SRGB": "ASTC_12x10_SRGB_BLOCK",
  "12x12_UNORM": "ASTC_12x12_UNORM_BLOCK",
  "12x12_SRGB": "ASTC_12x12_SRGB_BLOCK",
} satisfies Record<`${AstcBlock}_${AstcColor}`, Extract<VkFormat, `ASTC_${string}`>>;

const uncompressedValues = [
  "R8_UNORM",
  "R8_SRGB",
  "R8G8_UNORM",
  "R8G8_SRGB",
  "R8G8B8_UNORM",
  "R8G8B8_SRGB",
  "R8G8B8A8_UNORM",
  "R8G8B8A8_SRGB",
  "R16G16B16_SFLOAT",
  "R16G16B16A16_SFLOAT",
] as const satisfies readonly UncompressedFormat[];

const encodeValues = ["basis-lz", "uastc"] as const satisfies readonly EncodeCodec[];

const astcBlockValues = [
  "4x4",
  "5x4",
  "5x5",
  "6x5",
  "6x6",
  "8x5",
  "8x6",
  "8x8",
  "10x5",
  "10x6",
  "10x8",
  "10x10",
  "12x10",
  "12x12",
] as const satisfies readonly AstcBlock[];

const astcColorValues = ["UNORM", "SRGB"] as const satisfies readonly AstcColor[];

const modeValues = [
  "uncompressed",
  "astc",
  "codec",
] as const satisfies readonly FormatMode[];

const uncompressedOptions = uncompressedValues.map((value) => ({
  value,
  label: uncompressedLabels[value],
}));
const encodeOptions = encodeValues.map((value) => ({
  value,
  label: encodeLabels[value],
}));
const astcBlockOptions = astcBlockValues.map((value) => ({
  value,
  label: astcBlockLabels[value],
}));
const astcColorOptions = astcColorValues.map((value) => ({
  value,
  label: astcColorLabels[value],
}));
const modeOptions = modeValues.map((value) => ({
  value,
  label: modeLabels[value],
}));

const controlClass =
  "w-full rounded border border-line bg-surface px-2 py-1 text-sm text-ink";

function isAstcFormat(format: VkFormat): boolean {
  return format.startsWith("ASTC_");
}

function isUncompressedFormat(format: string): format is UncompressedFormat {
  return format in uncompressedLabels;
}

function isEncodeCodec(value: string): value is EncodeCodec {
  return value in encodeLabels;
}

function isAstcBlock(value: string): value is AstcBlock {
  return value in astcBlockLabels;
}

function isAstcColor(value: string): value is AstcColor {
  return value in astcColorLabels;
}

function isFormatMode(value: string): value is FormatMode {
  return value in modeLabels;
}

function parseAstc(format: VkFormat): { block: AstcBlock; color: AstcColor } {
  const match = /^ASTC_(\d+x\d+)_(UNORM|SRGB)_BLOCK$/.exec(format);
  if (match === null) {
    return { block: "6x6", color: "SRGB" };
  }
  const block = match[1];
  const color = match[2];
  if (!isAstcBlock(block) || !isAstcColor(color)) {
    return { block: "6x6", color: "SRGB" };
  }
  return { block, color };
}

function astcFormat(block: AstcBlock, color: AstcColor): VkFormat {
  return astcFormats[`${block}_${color}`];
}

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

function setMode(next: FormatMode): void {
  if (next === "uncompressed") {
    if (!isUncompressedFormat(store.options.format)) {
      store.options.format = "R8G8B8A8_SRGB";
    }
    store.options.encode = null;
    return;
  }
  if (next === "astc") {
    const parts = parseAstc(store.options.format);
    store.options.format = astcFormat(parts.block, parts.color);
    store.options.encode = null;
    return;
  }
  if (!isUncompressedFormat(store.options.format)) {
    store.options.format = "R8G8B8A8_SRGB";
  }
  if (store.options.encode === null) {
    store.options.encode = "basis-lz";
  }
}

function onModeChange(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLSelectElement)) {
    return;
  }
  if (isFormatMode(target.value)) {
    setMode(target.value);
  }
}

function onUncompressedFormat(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLSelectElement)) {
    return;
  }
  if (isUncompressedFormat(target.value)) {
    store.options.format = target.value;
    if (mode.value !== "codec") {
      store.options.encode = null;
    }
  }
}

function onAstcBlock(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLSelectElement)) {
    return;
  }
  if (isAstcBlock(target.value)) {
    store.options.format = astcFormat(target.value, astcParts.value.color);
    store.options.encode = null;
  }
}

function onAstcColor(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLSelectElement)) {
    return;
  }
  if (isAstcColor(target.value)) {
    store.options.format = astcFormat(astcParts.value.block, target.value);
    store.options.encode = null;
  }
}

function onEncode(event: Event): void {
  const target = event.target;
  if (!(target instanceof HTMLSelectElement)) {
    return;
  }
  if (target.value === "") {
    store.options.encode = null;
    return;
  }
  if (isEncodeCodec(target.value)) {
    if (!isUncompressedFormat(store.options.format)) {
      store.options.format = "R8G8B8A8_SRGB";
    }
    store.options.encode = target.value;
  }
}
</script>

<template>
  <Panel title="Format">
    <div class="field-row">
      <div class="label">
        Group
      </div>
      <div class="min-w-0 flex-1">
        <select
          :class="controlClass"
          :value="mode"
          @change="onModeChange"
        >
          <option
            v-for="option in modeOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </select>
      </div>
    </div>

    <OptionField
      label="Format"
      option-key="format"
      :state="fieldState('format')"
    >
      <select
        v-if="mode === 'uncompressed' || mode === 'codec'"
        :class="controlClass"
        :value="store.options.format"
        @change="onUncompressedFormat"
      >
        <optgroup :label="mode === 'codec' ? 'Uncompressed (with --encode)' : 'Uncompressed'">
          <option
            v-for="option in uncompressedOptions"
            :key="option.value"
            :value="option.value"
          >
            {{ option.label }}
          </option>
        </optgroup>
      </select>
      <div
        v-else
        class="flex flex-col gap-2"
      >
        <select
          :class="controlClass"
          :value="astcParts.block"
          aria-label="ASTC block size"
          @change="onAstcBlock"
        >
          <optgroup label="ASTC block size">
            <option
              v-for="option in astcBlockOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </optgroup>
        </select>
        <select
          :class="controlClass"
          :value="astcParts.color"
          aria-label="ASTC color space"
          @change="onAstcColor"
        >
          <optgroup label="sRGB / UNORM">
            <option
              v-for="option in astcColorOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </optgroup>
        </select>
      </div>
    </OptionField>

    <OptionField
      label="Encode"
      option-key="encode"
      :state="fieldState('encode')"
    >
      <select
        :class="controlClass"
        :value="store.options.encode ?? ''"
        :disabled="mode === 'astc'"
        @change="onEncode"
      >
        <option value="">
          None
        </option>
        <option
          v-for="option in encodeOptions"
          :key="option.value"
          :value="option.value"
        >
          {{ option.label }}
        </option>
      </select>
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
