/**
 * 4. The inspector viewer does what its controls say: Fit, Actual size, Zoom
 * In / Out (View menu), Ctrl+wheel, drag to pan, sticky zoom while scrubbing mips,
 * and Flip Y. The real frontend runs in Chromium against the real Rust commands.
 */
import type { Page } from "@playwright/test";
import { bridge, menu } from "../support/bridge";
import { decode } from "../support/ktx";
import { corpus } from "../support/paths";
import { readRgba } from "../support/pixels";
import { expect, test } from "../support/test";

type Box = { x: number; y: number; width: number; height: number };

async function open(page: Page, ipc: Parameters<typeof bridge>[1], file: string): Promise<void> {
  await bridge(page, ipc, "inspector", { inspector_current_file: () => corpus(file) });
  await page.goto("/inspector.html");
  await expect(image(page)).toBeVisible();
  await expect.poll(() => image(page).evaluate((img: HTMLImageElement) => img.naturalWidth)).toBeGreaterThan(0);
}

function frame(page: Page) {
  return page.locator(".viewer-frame");
}

function image(page: Page) {
  return frame(page).locator("img.viewer-img");
}

async function box(locator: ReturnType<typeof image>): Promise<Box> {
  const b = await locator.boundingBox();
  if (b === null) throw new Error("not rendered");
  return b;
}

/** The image is as large as fits (Fit keeps a small margin) and centred in the viewport. */
async function expectFit(page: Page): Promise<void> {
  await expect
    .poll(async () => {
      const f = await box(frame(page));
      const i = await box(image(page));
      const touches = Math.max(i.width / f.width, i.height / f.height) > 0.95;
      const inside = i.width <= f.width + 1 && i.height <= f.height + 1;
      const centred =
        Math.abs(i.x + i.width / 2 - (f.x + f.width / 2)) < 1.5 && Math.abs(i.y + i.height / 2 - (f.y + f.height / 2)) < 1.5;
      return touches && inside && centred;
    })
    .toBe(true);
}

async function expectWidth(page: Page, width: number): Promise<void> {
  await expect.poll(async () => Math.round((await box(image(page))).width)).toBe(width);
}

test.use({ viewport: { width: 1400, height: 900 } });

test("opens in Fit, and View → Fit restores it after zooming", async ({ page, ipc }) => {
  await open(page, ipc, "r8g8b8a8_srgb.ktx2");
  await expectFit(page);
  await menu(page, "zoom-in");
  await menu(page, "zoom-fit");
  await expectFit(page);
});

test("View → Actual size shows one texel per pixel, exactly the file's image", async ({ page, ipc }) => {
  await open(page, ipc, "r8g8b8a8_srgb.ktx2");
  await menu(page, "zoom-actual");
  await expectWidth(page, 256);
  await expect(frame(page)).toContainText("256×256 · 100%");

  const at = await box(image(page));
  // 1:1 only holds if the image lands on whole pixels; a half-pixel offset resamples it.
  expect.soft([at.x % 1, at.y % 1], "image origin on the pixel grid").toEqual([0, 0]);
  const shown = await readRgba(
    await page.screenshot({ clip: { x: Math.round(at.x), y: Math.round(at.y), width: 256, height: 256 }, animations: "disabled" }),
  );
  const expected = await decode(corpus("r8g8b8a8_srgb.ktx2"));
  // Opaque texels only; transparent ones show the checkerboard through.
  let worst = 0;
  for (let i = 0; i < expected.data.length; i += 4) {
    if (expected.data[i + 3] !== 255) continue;
    for (let c = 0; c < 3; c++) worst = Math.max(worst, Math.abs((shown.data[i + c] ?? 0) - (expected.data[i + c] ?? 0)));
  }
  expect(worst).toBeLessThanOrEqual(2);
});

test("Zoom In and Zoom Out step from Actual size", async ({ page, ipc }) => {
  await open(page, ipc, "r8g8b8a8_srgb.ktx2");
  await menu(page, "zoom-actual");
  await expectWidth(page, 256);
  await menu(page, "zoom-in");
  await expect.poll(async () => (await box(image(page))).width).toBeGreaterThan(256);
  await menu(page, "zoom-actual");
  await menu(page, "zoom-out");
  await expect.poll(async () => (await box(image(page))).width).toBeLessThan(256);
});

test("Actual size sticks while scrubbing mips: level 1 shows at half size", async ({ page, ipc }) => {
  await open(page, ipc, "mips-rgba8.ktx2");
  await menu(page, "zoom-actual");
  await expectWidth(page, 256);
  await page.locator("#inspector-mip").fill("1");
  await expect(page.getByText("Mip L1 · 128×128")).toBeVisible();
  await expectWidth(page, 128);
  await page.locator("#inspector-mip").fill("3");
  await expectWidth(page, 32);
});

test("Fit sticks while scrubbing mips", async ({ page, ipc }) => {
  await open(page, ipc, "mips-rgba8.ktx2");
  await expectFit(page);
  await page.locator("#inspector-mip").fill("2");
  await expect(page.getByText("Mip L2 · 64×64")).toBeVisible();
  await expectFit(page);
});

test("Ctrl+wheel zooms and drag pans", async ({ page, ipc }) => {
  await open(page, ipc, "r8g8b8a8_srgb.ktx2");
  await menu(page, "zoom-actual");
  await expectWidth(page, 256);
  const f = await box(frame(page));
  await page.mouse.move(f.x + f.width / 2, f.y + f.height / 2);
  await page.keyboard.down("Control");
  await page.mouse.wheel(0, -200);
  await page.keyboard.up("Control");
  await expect.poll(async () => (await box(image(page))).width).toBeGreaterThan(256);

  const before = await box(image(page));
  await page.mouse.down();
  await page.mouse.move(f.x + f.width / 2 + 60, f.y + f.height / 2 + 40, { steps: 5 });
  await page.mouse.up();
  await expect.poll(async () => {
    const after = await box(image(page));
    return [Math.round(after.x - before.x), Math.round(after.y - before.y)];
  }).toEqual([60, 40]);
});

test("Flip Y is off by default and its switch flips the image", async ({ page, ipc }) => {
  await open(page, ipc, "r8g8b8a8_srgb.ktx2");
  const transform = () => image(page).evaluate((img) => getComputedStyle(img).transform);
  await expect.poll(transform).toBe("none");
  await page.getByText("Flip Y").click();
  await expect.poll(transform).toBe("matrix(1, 0, 0, -1, 0, 0)");
});
