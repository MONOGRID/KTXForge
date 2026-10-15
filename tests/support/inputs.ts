/**
 * Deterministic synthetic inputs, generated before the suite runs.
 * Every image of a set (layers, faces, mips, slices) looks different from its
 * siblings, so a swapped or dropped subresource shows up as a PSNR collapse.
 */
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import sharp, { type Sharp } from "sharp";
import type { Rgba } from "./pixels";

function prng(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function clamp8(v: number): number {
  return Math.max(0, Math.min(255, Math.round(v)));
}

/**
 * Test chart: smooth ramps, a checkerboard quadrant (hard edges), a ring quadrant
 * (frequency sweep) and mild noise. `variant` rotates hue, phase and pattern.
 */
function chart(width: number, height: number, variant: number, alpha: boolean): Rgba {
  const random = prng(0x5eed + variant * 7919);
  const data = new Uint8Array(width * height * 4);
  const cx = width * 0.25;
  const cy = height * 0.75;
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const u = width > 1 ? x / (width - 1) : 0;
      const v = height > 1 ? y / (height - 1) : 0;
      let rgb = [255 * u, 255 * v, 127.5 + 127.5 * Math.sin((u + v + variant * 0.37) * Math.PI * 2)];
      if (u >= 0.5 && v < 0.5) {
        const on = ((x >> 3) + (y >> 3) + variant) & 1;
        rgb = on ? [230, 200 - variant * 15, 40 + variant * 20] : [20 + variant * 10, 30, 60];
      } else if (u < 0.5 && v >= 0.5) {
        const s = 0.5 + 0.5 * Math.sin(Math.hypot(x - cx, y - cy) * (0.4 + variant * 0.05));
        rgb = [255 * s, 128 + 100 * Math.cos(variant), 255 * (1 - s)];
      }
      const shift = variant % 3;
      const tinted = [rgb[shift % 3] ?? 0, rgb[(shift + 1) % 3] ?? 0, rgb[(shift + 2) % 3] ?? 0];
      const i = (y * width + x) * 4;
      for (let c = 0; c < 3; c++) {
        data[i + c] = clamp8((tinted[c] ?? 0) + (random() - 0.5) * 16);
      }
      let a = 255;
      if (alpha) {
        const d = Math.hypot(u - 0.5, v - 0.5);
        a = 255 * Math.min(1, Math.max(0, (0.6 - d) / 0.25));
        if (u > 0.8 && v > 0.8) a = 0;
      }
      data[i + 3] = clamp8(a);
    }
  }
  return { width, height, data };
}

/** Tangent-space normal map of bumps and ripples, unit length, RGB = XYZ. */
function normalMap(width: number, height: number): Rgba {
  const h = (x: number, y: number): number =>
    3 * Math.exp(-((x - 80) ** 2 + (y - 90) ** 2) / 600) +
    2 * Math.exp(-((x - 180) ** 2 + (y - 170) ** 2) / 900) +
    0.6 * Math.sin(x * 0.15) * Math.cos(y * 0.11);
  const data = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const dx = (h(x + 1, y) - h(x - 1, y)) * 2;
      const dy = (h(x, y + 1) - h(x, y - 1)) * 2;
      const len = Math.hypot(dx, dy, 1);
      const i = (y * width + x) * 4;
      data[i] = clamp8((-dx / len * 0.5 + 0.5) * 255);
      data[i + 1] = clamp8((-dy / len * 0.5 + 0.5) * 255);
      data[i + 2] = clamp8((1 / len * 0.5 + 0.5) * 255);
      data[i + 3] = 255;
    }
  }
  return { width, height, data };
}

/** Single-channel mask: soft disc, hard bars, ramp. */
function mask(width: number, height: number): Uint8Array {
  const data = new Uint8Array(width * height);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const d = Math.hypot(x - width * 0.35, y - height * 0.4) / (width * 0.3);
      let v = 255 * Math.max(0, 1 - d);
      if (x > width * 0.7) v = (x >> 2) & 1 ? 255 : 0;
      if (y > height * 0.85) v = (x / width) * 255;
      data[y * width + x] = clamp8(v);
    }
  }
  return data;
}

function rgbaSharp(image: Rgba): Sharp {
  return sharp(image.data, { raw: { width: image.width, height: image.height, channels: 4 } });
}

async function writePng(dir: string, name: string, image: Rgba, opaque = false): Promise<void> {
  const pipeline = opaque ? rgbaSharp(image).removeAlpha() : rgbaSharp(image);
  await pipeline.png({ compressionLevel: 9 }).toFile(join(dir, name));
}

const SIZES = { chart: 256, set: 64, cube: 64, mipBase: 64 } as const;

export function frameName(i: number): string {
  return `frame_${String(i + 1).padStart(2, "0")}.png`;
}

export function faceName(i: number): string {
  return `face_${String(i)}.png`;
}

function mipName(level: number): string {
  return `mip_${String(level)}.png`;
}

export function sliceName(i: number): string {
  return `slice_${String(i)}.png`;
}

const FRAME_COUNT = 4;
const SLICE_COUNT = 4;
const MIP_COUNT = Math.log2(SIZES.mipBase) + 1;

export async function generateInputs(dir: string): Promise<void> {
  await mkdir(dir, { recursive: true });
  const color = chart(SIZES.chart, SIZES.chart, 0, true);
  await writePng(dir, "color-rgba.png", color);
  await writePng(dir, "color-rgb.png", chart(SIZES.chart, SIZES.chart, 0, false), true);
  await writePng(dir, "normal.png", normalMap(SIZES.chart, SIZES.chart), true);
  await sharp(mask(SIZES.chart, SIZES.chart), {
    raw: { width: SIZES.chart, height: SIZES.chart, channels: 1 },
  })
    .png()
    .toFile(join(dir, "mask.png"));
  await writePng(dir, "odd-37x19.png", chart(37, 19, 1, true));
  // Large enough that a slow encode is still running when Cancel arrives.
  await writePng(dir, "big-rgba.png", chart(2048, 2048, 2, true));
  await rgbaSharp(color).toColourspace("rgb16").png().toFile(join(dir, "color-rgba16.png"));

  // Other containers the encoder converts before `ktx create`.
  await rgbaSharp(color).webp({ lossless: true, exact: true }).toFile(join(dir, "color-rgba.webp"));
  await rgbaSharp(color).tiff({ compression: "lzw" }).toFile(join(dir, "color-rgba.tiff"));
  await rgbaSharp(chart(SIZES.chart, SIZES.chart, 0, false))
    .removeAlpha()
    .jpeg({ quality: 92 })
    .toFile(join(dir, "photo.jpg"));

  // Names `ktx create` would misread: a comma splits the path list, `@` is a response file.
  await copyFile(join(dir, "color-rgba.png"), join(dir, "comma, name.png"));
  await copyFile(join(dir, "color-rgba.png"), join(dir, "@at-name.png"));

  for (let i = 0; i < FRAME_COUNT; i++) {
    await writePng(dir, frameName(i), chart(SIZES.set, SIZES.set, i + 1, true));
  }
  for (let i = 0; i < 6; i++) {
    await writePng(dir, faceName(i), chart(SIZES.cube, SIZES.cube, i + 1, false), true);
  }
  for (let level = 0; level < MIP_COUNT; level++) {
    const size = SIZES.mipBase >> level;
    await writePng(dir, mipName(level), chart(size, size, level + 1, true));
  }
  for (let i = 0; i < SLICE_COUNT; i++) {
    await writePng(dir, sliceName(i), chart(SIZES.set, SIZES.set, i + 3, true));
  }
  // Mixed sizes that must not be packed into one array.
  await writePng(dir, "frame_big.png", chart(SIZES.set * 2, SIZES.set * 2, 9, true));

  await writeFile(join(dir, "ramp.hdr"), radianceRamp(32, 16));

  // Small copies for the option sweep, where encode count matters more than size.
  const small = chart(SIZES.set, SIZES.set, 5, true);
  await writePng(dir, "small-rgba.png", small);
  await writePng(dir, "small-rgb.png", small, true);
  await sharp(mask(SIZES.set, SIZES.set), { raw: { width: SIZES.set, height: SIZES.set, channels: 1 } })
    .png()
    .toFile(join(dir, "small-gray.png"));
  await rgbaSharp(small).removeAlpha().jpeg({ quality: 90 }).toFile(join(dir, "small.jpg"));
}

/**
 * Flat (uncompressed) Radiance RGBE ramp from about 1/64 to about 48. No texel
 * starts with (2, 2) or is (1, 1, 1), so no reader mistakes it for RLE.
 */
function radianceRamp(width: number, height: number): Buffer {
  const rgbe = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4;
      const lead = 128 + ((x * 7 + y * 3) % 128);
      rgbe[i] = lead;
      rgbe[i + 1] = ((lead * (y + 1)) % 128) + 64;
      rgbe[i + 2] = ((lead * (31 - y)) % 96) + 16;
      rgbe[i + 3] = 122 + Math.floor((x / width) * 13);
    }
  }
  const header = `#?RADIANCE\nFORMAT=32-bit_rle_rgbe\n\n-Y ${String(height)} +X ${String(width)}\n`;
  return Buffer.concat([Buffer.from(header, "ascii"), Buffer.from(rgbe)]);
}
