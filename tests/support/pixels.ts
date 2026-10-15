import sharp from "sharp";

/** 8-bit RGBA, row-major, top-left origin. */
export type Rgba = { width: number; height: number; data: Uint8Array };

type Channel = 0 | 1 | 2 | 3;
const RGBA: readonly Channel[] = [0, 1, 2, 3];

/** Decode any image sharp reads (PNG, JPEG, grey, RGB) to RGBA. */
export async function readRgba(input: string | Buffer): Promise<Rgba> {
  const { data, info } = await sharp(input)
    .toColourspace("srgb")
    .ensureAlpha()
    .raw()
    .toBuffer({ resolveWithObject: true });
  return { width: info.width, height: info.height, data: new Uint8Array(data) };
}

export function pngOf(image: Rgba): Promise<Buffer> {
  return sharp(image.data, { raw: { width: image.width, height: image.height, channels: 4 } })
    .png({ compressionLevel: 9 })
    .toBuffer();
}

function assertSameSize(a: Rgba, b: Rgba): void {
  if (a.width !== b.width || a.height !== b.height) {
    throw new Error(`size mismatch: ${String(a.width)}x${String(a.height)} vs ${String(b.width)}x${String(b.height)}`);
  }
}

/** Peak signal-to-noise ratio in dB over `channels`; `Infinity` when identical. */
export function psnr(a: Rgba, b: Rgba, channels: readonly Channel[] = RGBA): number {
  assertSameSize(a, b);
  let sum = 0;
  let count = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    for (const c of channels) {
      const d = (a.data[i + c] ?? 0) - (b.data[i + c] ?? 0);
      sum += d * d;
      count += 1;
    }
  }
  const mse = sum / count;
  return mse === 0 ? Infinity : 10 * Math.log10((255 * 255) / mse);
}

/** Largest absolute difference over `channels`. */
export function maxAbsDiff(a: Rgba, b: Rgba, channels: readonly Channel[] = RGBA): number {
  assertSameSize(a, b);
  let max = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    for (const c of channels) {
      max = Math.max(max, Math.abs((a.data[i + c] ?? 0) - (b.data[i + c] ?? 0)));
    }
  }
  return max;
}
