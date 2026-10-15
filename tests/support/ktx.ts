/**
 * Independent oracle: Khronos' own `ktx` reads back what the app wrote.
 * Nothing here calls app code.
 */
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { z } from "zod";
import type { ImageSelection } from "../../src/bindings/ImageSelection";
import { ktxPath } from "./paths";
import { readRgba, type Rgba } from "./pixels";
import { run } from "./run";

const infoSchema = z.object({
  valid: z.boolean(),
  messages: z.array(z.object({ type: z.string(), message: z.string() })),
  header: z.object({
    vkFormat: z.string(),
    pixelWidth: z.number(),
    pixelHeight: z.number(),
    pixelDepth: z.number(),
    layerCount: z.number(),
    faceCount: z.number(),
    levelCount: z.number(),
    supercompressionScheme: z.string(),
  }),
  dataFormatDescriptor: z.object({
    blocks: z.array(
      z.object({
        colorModel: z.string().optional(),
        transferFunction: z.string().optional(),
        colorPrimaries: z.string().optional(),
        samples: z.array(z.object({ channelType: z.string() })).default([]),
      }),
    ),
  }),
  keyValueData: z.record(z.string(), z.unknown()).default({}),
});

export type KtxInfo = z.infer<typeof infoSchema>;

export async function ktxInfo(file: string): Promise<KtxInfo> {
  const out = await run(ktxPath(), ["info", "--format", "json", file]);
  const parsed: unknown = JSON.parse(out.stdout);
  return infoSchema.parse(parsed);
}

/** `ktx validate` verdict; the text report is the failure message. */
export async function ktxValidate(file: string): Promise<{ valid: boolean; report: string }> {
  const out = await run(ktxPath(), ["validate", file]);
  return { valid: out.code === 0, report: `${out.stdout}${out.stderr}`.trim() };
}

/** Non-zero size of a dimension the way `ktx` counts it (0 means "not an array / 2D"). */
export function counts(info: KtxInfo): { levels: number; faces: number; layers: number; depth: number } {
  return {
    levels: Math.max(1, info.header.levelCount),
    faces: Math.max(1, info.header.faceCount),
    layers: Math.max(1, info.header.layerCount),
    depth: Math.max(1, info.header.pixelDepth),
  };
}

function isBasis(info: KtxInfo): boolean {
  return info.header.vkFormat === "VK_FORMAT_UNDEFINED";
}

export function isFloat(info: KtxInfo): boolean {
  return /FLOAT/.test(info.header.vkFormat);
}

/** Channels a BasisLZ / UASTC payload carries, from its DFD sample types. */
function basisChannels(info: KtxInfo): number {
  const types = (info.dataFormatDescriptor.blocks[0]?.samples ?? []).map((s) => s.channelType);
  if (types.some((t) => t.endsWith("_RGBA") || t.endsWith("_AAA"))) return 4;
  if (types.some((t) => t.endsWith("_RGB"))) return 3;
  if (types.some((t) => t.endsWith("_RRRG") || t.endsWith("_GGG") || t.endsWith("_RG"))) return 2;
  return 1;
}

const TRANSCODE_TARGETS = ["r8", "rg8", "rgb8", "rgba8"] as const;

/** Only dimensions the file has; v4.4.2 rejects `--layer 0` on a non-array texture. */
function selectionArgs(info: KtxInfo, selection: Partial<ImageSelection>): string[] {
  const args: string[] = [];
  if (info.header.levelCount > 1) args.push("--level", String(selection.level ?? 0));
  if (info.header.faceCount > 1) args.push("--face", String(selection.face ?? 0));
  if (info.header.layerCount > 0) args.push("--layer", String(selection.layer ?? 0));
  if (info.header.pixelDepth > 0) args.push("--depth", String(selection.depthSlice ?? 0));
  return args;
}

async function extract(file: string, args: string[], name: string): Promise<Buffer> {
  const dir = await mkdtemp(join(tmpdir(), "ktxforge-e2e-"));
  try {
    const out = join(dir, name);
    const result = await run(ktxPath(), ["extract", ...args, file, out]);
    if (result.code !== 0) {
      throw new Error(`ktx extract ${args.join(" ")} failed (${String(result.code)}): ${result.stderr}`);
    }
    return await readFile(out);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

/**
 * One LDR subresource as `ktx extract` writes it, decoded to RGBA. BasisLZ / UASTC
 * transcode to the unpacked format with the payload's channel count.
 */
export async function decode(file: string, selection: Partial<ImageSelection> = {}): Promise<Rgba> {
  const info = await ktxInfo(file);
  const target = TRANSCODE_TARGETS[basisChannels(info) - 1] ?? "rgba8";
  const transcode = isBasis(info) ? ["--transcode", target] : [];
  return readRgba(await extract(file, [...transcode, ...selectionArgs(info, selection)], "image.png"));
}
