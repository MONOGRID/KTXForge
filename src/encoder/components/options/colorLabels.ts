import type { ColorPrimaries } from "../../../bindings/ColorPrimaries";
import type { ConvertPrimaries } from "../../../bindings/ConvertPrimaries";
import type { ConvertTransferFunction } from "../../../bindings/ConvertTransferFunction";
import type { TexcoordOrigin } from "../../../bindings/TexcoordOrigin";
import type { TransferFunction } from "../../../bindings/TransferFunction";

export const transferFunctionLabels = {
  linear: "linear",
  srgb: "srgb",
  srgb_eotf: "srgb_eotf",
  scrgb: "scrgb",
  scrgb_eotf: "scrgb_eotf",
  itu: "itu",
  itu_oetf: "itu_oetf",
  bt601: "bt601",
  bt601_oetf: "bt601_oetf",
  bt709: "bt709",
  bt709_oetf: "bt709_oetf",
  bt2020: "bt2020",
  bt2020_oetf: "bt2020_oetf",
  smpte170m: "smpte170m",
  smpte170m_oetf: "smpte170m_oetf",
  smpte170m_eotf: "smpte170m_eotf",
  ntsc: "ntsc",
  ntsc_eotf: "ntsc_eotf",
  slog: "slog",
  slog_oetf: "slog_oetf",
  slog2: "slog2",
  slog2_oetf: "slog2_oetf",
  bt1886: "bt1886",
  bt1886_eotf: "bt1886_eotf",
  hlg_oetf: "hlg_oetf",
  hlg_eotf: "hlg_eotf",
  pq_oetf: "pq_oetf",
  pq_eotf: "pq_eotf",
  dcip3: "dcip3",
  dcip3_eotf: "dcip3_eotf",
  pal_oetf: "pal_oetf",
  pal625_eotf: "pal625_eotf",
  st240: "st240",
  st240_oetf: "st240_oetf",
  st240_eotf: "st240_eotf",
  acescc: "acescc",
  acescc_oetf: "acescc_oetf",
  acescct: "acescct",
  acescct_oetf: "acescct_oetf",
  adobergb: "adobergb",
  adobergb_eotf: "adobergb_eotf",
} satisfies Record<TransferFunction, string>;

export const convertTfLabels = {
  linear: "linear",
  srgb: "srgb",
  srgb_eotf: "srgb_eotf",
  scrgb: "scrgb",
  scrgb_eotf: "scrgb_eotf",
} satisfies Record<ConvertTransferFunction, string>;

export const colorPrimariesLabels = {
  none: "none",
  bt709: "bt709",
  srgb: "srgb",
  bt601_ebu: "bt601_ebu",
  bt601_smpte: "bt601_smpte",
  bt2020: "bt2020",
  ciexyz: "ciexyz",
  aces: "aces",
  acescc: "acescc",
  ntsc1953: "ntsc1953",
  pal525: "pal525",
  displayp3: "displayp3",
  adobergb: "adobergb",
} satisfies Record<ColorPrimaries, string>;

export const convertPrimariesLabels = {
  bt709: "bt709",
  srgb: "srgb",
  bt601_ebu: "bt601_ebu",
  bt601_smpte: "bt601_smpte",
  bt2020: "bt2020",
  ciexyz: "ciexyz",
  aces: "aces",
  acescc: "acescc",
  ntsc1953: "ntsc1953",
  pal525: "pal525",
  displayp3: "displayp3",
  adobergb: "adobergb",
} satisfies Record<ConvertPrimaries, string>;

export const texcoordOriginLabels = {
  "top-left": "top-left",
  "bottom-left": "bottom-left",
  "top-left-front": "top-left-front",
  "top-left-back": "top-left-back",
  "bottom-left-front": "bottom-left-front",
  "bottom-left-back": "bottom-left-back",
} satisfies Record<TexcoordOrigin, string>;
