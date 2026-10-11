/**
 * Build src-tauri/icons from the brand masters in assets/.
 *
 * - assets/icon-simple.png → sizes < 128 (taskbar / tray / small tiles)
 * - assets/icon-full.png   → sizes ≥ 128 (dock / installers / store)
 *
 * Requires (install temporarily if missing):
 *   npm install --no-save sharp to-ico @fiahfy/icns
 */
import { mkdir, writeFile, copyFile } from "node:fs/promises";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, "..");

function loadDep(name) {
  try {
    return require(name);
  } catch {
    console.error(
      `Missing dependency "${name}". Run:\n  npm install --no-save sharp to-ico @fiahfy/icns`,
    );
    process.exit(1);
  }
}

const sharp = loadDep("sharp");
const toIco = loadDep("to-ico");
const { Icns, IcnsImage } = loadDep("@fiahfy/icns");

const SIMPLE = path.join(root, "assets", "icon-simple.png");
const FULL = path.join(root, "assets", "icon-full.png");
const OUT = path.join(root, "src-tauri", "icons");
const PUBLIC = path.join(root, "public");

/** Sizes below this use the simple mark; this size and above use the full mark. */
const FULL_FROM = 128;

const PNG_FILES = [
  ["32x32.png", 32],
  ["128x128.png", 128],
  ["128x128@2x.png", 256],
  ["icon.png", 512],
  ["StoreLogo.png", 50],
  ["Square30x30Logo.png", 30],
  ["Square44x44Logo.png", 44],
  ["Square71x71Logo.png", 71],
  ["Square89x89Logo.png", 89],
  ["Square107x107Logo.png", 107],
  ["Square142x142Logo.png", 142],
  ["Square150x150Logo.png", 150],
  ["Square284x284Logo.png", 284],
  ["Square310x310Logo.png", 310],
];

/** ICO layers: 32px first for crisp Tauri-dev window icons. */
const ICO_SIZES = [32, 16, 24, 48, 64, 256];

/** Modern PNG-in-ICNS types (matches typical `tauri icon` coverage). */
const ICNS_ENTRIES = [
  ["icp4", 16],
  ["icp5", 32],
  ["icp6", 64],
  ["ic07", 128],
  ["ic08", 256],
  ["ic09", 512],
  ["ic10", 1024],
  ["ic11", 32],
  ["ic12", 64],
  ["ic13", 256],
  ["ic14", 512],
];

function sourceFor(size) {
  return size < FULL_FROM ? SIMPLE : FULL;
}

async function pngBuffer(size) {
  return sharp(sourceFor(size))
    .resize(size, size, {
      fit: "fill",
      kernel: sharp.kernel.lanczos3,
    })
    .ensureAlpha()
    .png({ compressionLevel: 9 })
    .toBuffer();
}

async function main() {
  await mkdir(OUT, { recursive: true });
  await mkdir(PUBLIC, { recursive: true });

  const cache = new Map();
  async function cached(size) {
    const hit = cache.get(size);
    if (hit) return hit;
    const buf = await pngBuffer(size);
    cache.set(size, buf);
    return buf;
  }

  for (const [name, size] of PNG_FILES) {
    const buf = await cached(size);
    await writeFile(path.join(OUT, name), buf);
    console.log(`  ${name} (${size}px ← ${size < FULL_FROM ? "simple" : "full"})`);
  }

  const ico = await toIco(await Promise.all(ICO_SIZES.map((s) => cached(s))));
  await writeFile(path.join(OUT, "icon.ico"), ico);
  console.log(
    `  icon.ico (${ICO_SIZES.map((s) => `${s}:${s < FULL_FROM ? "S" : "F"}`).join(" ")})`,
  );

  const icns = new Icns();
  for (const [osType, size] of ICNS_ENTRIES) {
    icns.append(IcnsImage.fromPNG(await cached(size), osType));
  }
  await writeFile(path.join(OUT, "icon.icns"), icns.data);
  console.log("  icon.icns");

  await copyFile(path.join(OUT, "icon.ico"), path.join(PUBLIC, "favicon.ico"));
  await writeFile(path.join(PUBLIC, "favicon-32.png"), await cached(32));
  await writeFile(path.join(PUBLIC, "apple-touch-icon.png"), await cached(180));
  console.log("  public/favicon.ico, favicon-32.png, apple-touch-icon.png");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
