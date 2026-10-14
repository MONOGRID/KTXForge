/**
 * The options rail as data: where each control (by `OptionKey`) writes in
 * `CreateOptions`, and which values a user can give it.
 */
import type { OptionKey } from "../../src/bindings/OptionKey";
import type { Json } from "./ipc";
import type { EncoderSession, Field } from "./session";

export type Pick = (field: Field, current: Json | undefined, random: () => number) => Json;

export function oneOf<T>(values: readonly T[], random: () => number): T {
  const value = values[Math.floor(random() * values.length)];
  if (value === undefined) throw new Error("oneOf: empty list");
  return value;
}

const bool: Pick = (_field, current) => current !== true;
const numbers =
  (...values: number[]): Pick =>
  (_field, _current, random) =>
    oneOf(values, random);
/** Enum select: one of the offered choices; `nullable` adds the empty item. */
const choice =
  (nullable: boolean): Pick =>
  (field, _current, random) => {
    const values: Json[] = field.choices.map((c) => c.value);
    if (nullable) values.push(null);
    return values.length > 0 ? oneOf(values, random) : null;
  };
const text =
  (...values: string[]): Pick =>
  (_field, _current, random) =>
    oneOf(values, random);

/**
 * Where each control writes in `CreateOptions`, and what a user can put there
 * (numeric values stay inside each input's `min` / `max`).
 */
const CONTROLS: Partial<Record<OptionKey, { path: string[]; pick: Pick }>> = {
  "1d": { path: ["container", "oneD"], pick: bool },
  "generate-mipmap": { path: ["container", "generateMipmap"], pick: bool },
  "runtime-mipmap": { path: ["container", "runtimeMipmap"], pick: bool },
  levels: { path: ["container", "levels"], pick: numbers(1, 2, 3, 5, 7) },
  width: { path: ["container", "width"], pick: numbers(1, 16, 37, 100, 128) },
  height: { path: ["container", "height"], pick: numbers(1, 16, 37, 100, 128) },
  scale: { path: ["container", "scale"], pick: numbers(0.25, 0.5, 1, 1.5, 2) },
  "mipmap-filter": { path: ["filter", "mipmapFilter"], pick: choice(false) },
  "mipmap-filter-scale": { path: ["filter", "mipmapFilterScale"], pick: numbers(0.5, 1, 2) },
  "mipmap-wrap": { path: ["filter", "mipmapWrap"], pick: choice(false) },
  "assign-tf": { path: ["color", "assignTf"], pick: choice(true) },
  "convert-tf": { path: ["color", "convertTf"], pick: choice(true) },
  "assign-primaries": { path: ["color", "assignPrimaries"], pick: choice(true) },
  "convert-primaries": { path: ["color", "convertPrimaries"], pick: choice(true) },
  "assign-texcoord-origin": { path: ["color", "assignTexcoordOrigin"], pick: choice(true) },
  "convert-texcoord-origin": { path: ["color", "convertTexcoordOrigin"], pick: choice(true) },
  "fail-on-color-conversions": { path: ["color", "failOnColorConversions"], pick: bool },
  "warn-on-color-conversions": { path: ["color", "warnOnColorConversions"], pick: bool },
  "no-warn-on-color-conversions": { path: ["color", "noWarnOnColorConversions"], pick: bool },
  "fail-on-origin-changes": { path: ["color", "failOnOriginChanges"], pick: bool },
  "warn-on-origin-changes": { path: ["color", "warnOnOriginChanges"], pick: bool },
  swizzle: { path: ["color", "swizzle"], pick: text("rgba", "bgra", "rrr1", "rg01", "000r") },
  "input-swizzle": { path: ["color", "inputSwizzle"], pick: text("rgba", "bgra", "rrr1", "rg01", "000r") },
  normalize: { path: ["color", "normalize"], pick: bool },
  "normal-mode": { path: ["color", "normalMode"], pick: bool },
  format: { path: ["format"], pick: choice(false) },
  encode: { path: ["encode"], pick: choice(true) },
  qlevel: { path: ["basisLz", "qlevel"], pick: numbers(1, 64, 128, 255) },
  clevel: { path: ["basisLz", "clevel"], pick: numbers(0, 1, 3, 6) },
  "max-endpoints": { path: ["basisLz", "maxEndpoints"], pick: numbers(1, 256, 16128) },
  "max-selectors": { path: ["basisLz", "maxSelectors"], pick: numbers(1, 256, 16128) },
  "endpoint-rdo-threshold": { path: ["basisLz", "endpointRdoThreshold"], pick: numbers(1, 1.25, 3) },
  "selector-rdo-threshold": { path: ["basisLz", "selectorRdoThreshold"], pick: numbers(1, 1.25, 3) },
  "no-endpoint-rdo": { path: ["basisLz", "noEndpointRdo"], pick: bool },
  "no-selector-rdo": { path: ["basisLz", "noSelectorRdo"], pick: bool },
  "uastc-quality": { path: ["uastcLdr", "uastcQuality"], pick: numbers(0, 1, 2, 4) },
  "uastc-rdo": { path: ["uastcLdr", "uastcRdo"], pick: bool },
  "uastc-rdo-l": { path: ["uastcLdr", "uastcRdoL"], pick: numbers(0.001, 0.5, 1, 10) },
  "uastc-rdo-d": { path: ["uastcLdr", "uastcRdoD"], pick: numbers(64, 4096, 65536) },
  "uastc-rdo-b": { path: ["uastcLdr", "uastcRdoB"], pick: numbers(1, 10, 300) },
  "uastc-rdo-s": { path: ["uastcLdr", "uastcRdoS"], pick: numbers(0.01, 18, 65536) },
  "uastc-rdo-f": { path: ["uastcLdr", "uastcRdoF"], pick: bool },
  "uastc-rdo-m": { path: ["uastcLdr", "uastcRdoM"], pick: bool },
  "astc-quality": { path: ["astc", "astcQuality"], pick: choice(false) },
  "astc-perceptual": { path: ["astc", "astcPerceptual"], pick: bool },
  threads: { path: ["encoder", "threads"], pick: numbers(1, 2, 4) },
  "no-sse": { path: ["encoder", "noSse"], pick: bool },
  "compare-ssim": { path: ["encoder", "compareSsim"], pick: bool },
  "compare-psnr": { path: ["encoder", "comparePsnr"], pick: bool },
  // Supercompression is one scheme select plus a level input.
  zstd: {
    path: ["supercompression"],
    pick: (_f, _c, random) => oneOf<Json>([{ kind: "zstd", level: oneOf([1, 3, 10, 18, 22], random) }, { kind: "none" }], random),
  },
  zlib: {
    path: ["supercompression"],
    pick: (_f, _c, random) => oneOf<Json>([{ kind: "zlib", level: oneOf([1, 5, 9], random) }, { kind: "none" }], random),
  },
};

export function controlFor(key: string): { path: string[]; pick: Pick } | undefined {
  return Object.entries(CONTROLS).find(([k]) => k === key)?.[1];
}

export function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

/**
 * Change `key` the way a user does: only while the rail shows it. Throws when the
 * control is hidden, since a user could not make that edit.
 */
export async function edit(session: EncoderSession, key: OptionKey, value: Json): Promise<void> {
  const control = CONTROLS[key];
  if (control === undefined) throw new Error(`no control mapped for ${key}`);
  const field = (await session.report()).fields.find((f) => f.key === key);
  if (field?.enabled !== true) throw new Error(`${key} is not shown in the options rail:\n${session.describe()}`);
  await session.set(key, control.path, value);
}
