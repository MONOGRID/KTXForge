import type { EncodeCodec } from "../../../bindings/EncodeCodec";
import type { VkFormat } from "../../../bindings/VkFormat";

export type FormatMode = "uncompressed" | "astc" | "codec";

export type AstcBlock =
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

export type AstcColor = "UNORM" | "SRGB";

export type UncompressedFormat =
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

export const uncompressedLabels = {
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

export const encodeLabels = {
  "basis-lz": "BasisLZ",
  uastc: "UASTC",
} satisfies Record<EncodeCodec, string>;

export const astcBlockLabels = {
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

export const astcColorLabels = {
  UNORM: "UNORM",
  SRGB: "sRGB",
} satisfies Record<AstcColor, string>;

export const modeLabels = {
  uncompressed: "Uncompressed",
  astc: "ASTC",
  codec: "Codec (--encode)",
} satisfies Record<FormatMode, string>;

export const modeHelps = {
  uncompressed: "Pick an uncompressed Vulkan format only (no --encode).",
  astc: "ASTC block format. Encoder quality options appear in the ASTC panel.",
  codec: "Uncompressed format plus BasisLZ or UASTC via --encode.",
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

export const encodeValues = ["basis-lz", "uastc"] as const satisfies readonly EncodeCodec[];

export const astcBlockValues = [
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

export const modeValues = [
  "uncompressed",
  "astc",
  "codec",
] as const satisfies readonly FormatMode[];

export function isAstcFormat(format: VkFormat): boolean {
  return format.startsWith("ASTC_");
}

export function isUncompressedFormat(format: string): format is UncompressedFormat {
  return format in uncompressedLabels;
}

export function isEncodeCodec(value: string): value is EncodeCodec {
  return value in encodeLabels;
}

export function isAstcBlock(value: string): value is AstcBlock {
  return value in astcBlockLabels;
}

export function isAstcColor(value: string): value is AstcColor {
  return value in astcColorLabels;
}

export function isFormatMode(value: string): value is FormatMode {
  return value in modeLabels;
}

export function parseAstc(format: VkFormat): { block: AstcBlock; color: AstcColor } {
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

export function astcFormat(block: AstcBlock, color: AstcColor): VkFormat {
  return astcFormats[`${block}_${color}`];
}
