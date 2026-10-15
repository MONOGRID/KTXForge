/**
 * KTX files for the reading tests, written by Khronos' own `ktx create` (not by
 * the app) across formats, codecs and shapes, plus the checked-in fixtures.
 */
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { frameName, faceName, sliceName } from "./inputs";
import { run } from "./run";

type Recipe = { name: string; args: string[]; inputs: string[] };

const color = ["color-rgba.png"];
const opaque = ["color-rgb.png"];
const grey = ["mask.png"];
const frames = [0, 1, 2, 3].map(frameName);
const faces = [0, 1, 2, 3, 4, 5].map(faceName);
const slices = [0, 1, 2, 3].map(sliceName);

function uncompressed(format: string, inputs: string[]): Recipe {
  return { name: format.toLowerCase(), args: ["--format", format], inputs };
}

const RECIPES: Recipe[] = [
  ...["R8_UNORM", "R8_SRGB"].map((f) => uncompressed(f, grey)),
  ...["R8G8_UNORM", "R8G8_SRGB", "R8G8B8_UNORM", "R8G8B8_SRGB", "B8G8R8_SRGB"].map((f) => uncompressed(f, opaque)),
  ...[
    "R8G8B8A8_UNORM",
    "R8G8B8A8_SRGB",
    "B8G8R8A8_UNORM",
    "B8G8R8A8_SRGB",
    "R4G4B4A4_UNORM_PACK16",
    "R5G6B5_UNORM_PACK16",
    "A1R5G5B5_UNORM_PACK16",
  ].map((f) => uncompressed(f, color)),
  ...["A2B10G10R10_UNORM_PACK32", "R16G16B16A16_UNORM"].map((f) => uncompressed(f, ["color-rgba16.png"])),
  ...["4x4", "6x6", "8x8", "12x12"].map((b) => uncompressed(`ASTC_${b}_SRGB_BLOCK`, color)),
  uncompressed("ASTC_5x4_UNORM_BLOCK", color),
  ...["R16_SFLOAT", "R16G16B16A16_SFLOAT", "R32G32B32A32_SFLOAT", "B10G11R11_UFLOAT_PACK32", "E5B9G9R9_UFLOAT_PACK32"].map(
    (f) => uncompressed(f, ["ramp.exr"]),
  ),
  { name: "uastc", args: ["--format", "R8G8B8A8_SRGB", "--encode", "uastc"], inputs: color },
  { name: "uastc-zstd-mips", args: ["--format", "R8G8B8A8_SRGB", "--encode", "uastc", "--zstd", "18", "--generate-mipmap"], inputs: color },
  { name: "uastc-r8", args: ["--format", "R8_UNORM", "--encode", "uastc"], inputs: grey },
  { name: "uastc-rg8", args: ["--format", "R8G8_UNORM", "--encode", "uastc"], inputs: opaque },
  { name: "basislz-rgba-mips", args: ["--format", "R8G8B8A8_SRGB", "--encode", "basis-lz", "--generate-mipmap"], inputs: color },
  { name: "basislz-rgb", args: ["--format", "R8G8B8_SRGB", "--encode", "basis-lz"], inputs: opaque },
  { name: "basislz-r8", args: ["--format", "R8_UNORM", "--encode", "basis-lz"], inputs: grey },
  { name: "zlib", args: ["--format", "R8G8B8A8_SRGB", "--zlib", "6"], inputs: color },
  { name: "mips-rgba8", args: ["--format", "R8G8B8A8_SRGB", "--generate-mipmap"], inputs: color },
  { name: "array4", args: ["--format", "R8G8B8A8_SRGB", "--layers", "4"], inputs: frames },
  { name: "array4-uastc-mips", args: ["--format", "R8G8B8A8_SRGB", "--layers", "4", "--encode", "uastc", "--generate-mipmap"], inputs: frames },
  { name: "cubemap", args: ["--format", "R8G8B8_SRGB", "--cubemap"], inputs: faces },
  { name: "cubemap-basislz-mips", args: ["--format", "R8G8B8_SRGB", "--cubemap", "--encode", "basis-lz", "--generate-mipmap"], inputs: faces },
  { name: "volume4", args: ["--format", "R8G8B8A8_SRGB", "--depth", "4"], inputs: slices },
  { name: "odd-astc", args: ["--format", "ASTC_6x6_SRGB_BLOCK"], inputs: ["odd-37x19.png"] },
  { name: "bottom-left", args: ["--format", "R8G8B8A8_SRGB", "--assign-texcoord-origin", "bottom-left"], inputs: color },
];

/** Checked-in files, including a KTX1 file and broken ones. */
const FIXTURES = [
  "anim.ktx2",
  "array4_mips.ktx2",
  "astc_6x6_srgb.ktx2",
  "basis_lz.ktx2",
  "cubemap.ktx2",
  "hdr_uastc.ktx2",
  "legacy.ktx",
  "rgba8_srgb.ktx2",
  "uastc_zstd.ktx2",
];

/** Every corpus file name, known before setup writes them. */
export const CORPUS = [...RECIPES.map((r) => `${r.name}.ktx2`), ...FIXTURES.map((f) => `fixture-${f}`)];

export const BROKEN_FIXTURES = ["truncated.ktx2", "not_ktx.ktx2"];

/** Write the corpus into `dir`; returns what `ktx create` refused (logged, not fatal). */
export async function generateCorpus(ktx: string, inputs: string, fixtures: string, dir: string): Promise<string[]> {
  await mkdir(dir, { recursive: true });
  const refused: string[] = [];
  await Promise.all(
    RECIPES.map(async (recipe) => {
      const out = join(dir, `${recipe.name}.ktx2`);
      const result = await run(ktx, ["create", ...recipe.args, ...recipe.inputs.map((i) => join(inputs, i)), out]);
      if (result.code !== 0) refused.push(`${recipe.name}: ${result.stderr.trim()}`);
    }),
  );
  for (const name of FIXTURES) {
    await copyFile(join(fixtures, name), join(dir, `fixture-${name}`));
  }
  await writeFile(join(dir, "refused.txt"), refused.join("\n"));
  return refused;
}

