import type { OptionKey } from "../bindings/OptionKey";

/** Short tooltips from ktx create docs / requirements. */
const optionHelp = {
  "1d": "Create a 1D texture.",
  cubemap:
    "Cubemap faces in order +X, −X, +Y, −Y, +Z, −Z. Origin must be top-left.",
  layers: "Array texture layer count. Also the frame count for animation.",
  depth: "3D texture depth. Requires a front/back texture-coordinate origin.",
  levels:
    "Mip level count. With generate-mipmap, builds this many levels; otherwise this many input images are required.",
  "generate-mipmap":
    "Generate the mipmap chain during create. Full chain if levels is omitted. Not valid for SINT, UINT, 3D, or raw.",
  "runtime-mipmap":
    "Mark the texture for client-side mipmap generation later. Mutually exclusive with generate-mipmap.",
  width: "Output width. Required for raw; otherwise resamples and enables mipmap filter options.",
  height:
    "Output height. Required for raw; otherwise resamples and enables mipmap filter options.",
  scale: "Scale images as they are loaded. Not valid with raw. Enables mipmap filter options.",
  raw: "Treat input as raw. Width and height are required.",
  "mipmap-filter":
    "Filter used when generating mipmaps or resampling. Default lanczos4.",
  "mipmap-filter-scale": "Filter scale for mipmap/resample. Default 1.0.",
  "mipmap-wrap":
    "How to sample pixels near image boundaries when filtering. Default clamp.",
  "assign-tf":
    "Force the created texture transfer function, ignoring the input file(s).",
  "convert-tf":
    "Convert input to linear or sRGB. Not valid with raw.",
  "assign-primaries": "Force the created texture color primaries.",
  "convert-primaries":
    "Convert input primaries. Not valid with raw or when assigned primaries are none.",
  "assign-texcoord-origin":
    "Force texture-coordinate origin (top/bottom-left, plus front/back when depth is set).",
  "convert-texcoord-origin":
    "Convert origin (vertical flip only). Not valid with raw. Cubemaps must stay top-left.",
  "fail-on-color-conversions": "Error if any color conversion would run.",
  "warn-on-color-conversions":
    "Warn on every conversion, including visually lossless ones.",
  "no-warn-on-color-conversions":
    "Silence conversion warnings. Overrides the warn flag.",
  "fail-on-origin-changes": "Error if an origin flip is required.",
  "warn-on-origin-changes": "Warn when an origin changes.",
  swizzle: "KTX swizzle metadata. Four characters from rgba01.",
  "input-swizzle": "Pre-swizzle input channels. Four characters from rgba01.",
  normalize:
    "Normalize input normals to unit length. Linear, 2+ components. Not valid with raw.",
  "normal-mode":
    "Normal-map encode mode. RGB(A) XYZ becomes X+Y in RGB+A. For BasisLZ disables endpoint and selector RDO.",
  format: "Vulkan format written into the KTX2 file.",
  encode: "Encode with BasisLZ or UASTC after create.",
  qlevel:
    "BasisLZ quality 1–255. Lower is smaller/faster/lower quality. Default 128 when endpoints and selectors are unset.",
  clevel: "BasisLZ compression level 0–6 (default 1). Speed versus quality.",
  "max-endpoints":
    "BasisLZ max endpoints 1–16128. Takes effect only together with max-selectors.",
  "max-selectors": "BasisLZ max selectors 1–16128.",
  "endpoint-rdo-threshold":
    "BasisLZ endpoint RDO threshold (default 1.25, useful ~1.0–3.0). Overrides qlevel.",
  "selector-rdo-threshold":
    "BasisLZ selector RDO threshold (default 1.25, useful ~1.0–3.0).",
  "no-endpoint-rdo": "Disable BasisLZ endpoint RDO (on by default).",
  "no-selector-rdo": "Disable BasisLZ selector RDO (on by default).",
  "uastc-quality":
    "UASTC quality 0 (fastest) through 4 (very slow). Default 1.",
  "uastc-rdo": "Enable the UASTC RDO post-process.",
  "uastc-rdo-l": "UASTC RDO lambda 0.001–10 (default 1.0). Normals: try 0.25–0.75.",
  "uastc-rdo-d": "UASTC RDO dictionary size 64–65536 bytes (default 4096).",
  "uastc-rdo-b":
    "UASTC RDO smooth-block error scale 1–300 (default 10). 1 disables.",
  "uastc-rdo-s":
    "UASTC RDO smooth-block standard deviation 0.01–65536 (default 18).",
  "uastc-rdo-f": "Do not favor simpler UASTC modes during RDO.",
  "uastc-rdo-m": "Disable UASTC RDO multithreading.",
  "astc-quality":
    "ASTC search quality: fastest, fast, medium (default), thorough, exhaustive.",
  "astc-perceptual":
    "Use perceptual error instead of RMS. For normal maps and RGB only.",
  threads:
    "Encoder thread count. Default is hardware concurrency for BasisLZ, or 1.",
  "no-sse": "Forbid SSE on BasisLZ and UASTC compressors.",
  "compare-ssim": "Compute SSIM after encode (BasisLZ, UASTC, or ASTC).",
  "compare-psnr": "Compute PSNR after encode (BasisLZ, UASTC, or ASTC).",
  zstd: "Zstd supercompression level 1–22. Not valid with BasisLZ. Recommended with UASTC.",
  zlib: "Zlib supercompression level 1–9. Not valid with BasisLZ.",
} as const satisfies Record<OptionKey, string>;

export function helpFor(key: OptionKey): string {
  return optionHelp[key];
}
