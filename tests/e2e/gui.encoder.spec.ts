/**
 * 4. The encoder window, end to end in Chromium against the real Rust commands:
 * dropped files become outputs, Encode all + Save write a valid KTX2 whose image
 * the wipe shows, a bad pack keeps Encode disabled, and View zoom works here too.
 */
import { existsSync } from "node:fs";
import type { Page } from "@playwright/test";
import { z } from "zod";
import { bridge, menu } from "../support/bridge";
import { unwrap, type Ipc } from "../support/ipc";
import { decode, ktxValidate } from "../support/ktx";
import { input } from "../support/paths";
import { maxAbsDiff, readRgba } from "../support/pixels";
import { expect, test } from "../support/test";

const saveArgs = z.object({ jobId: z.string() });

/** Open the encoder with `files` dropped on it; Save… writes to `saveTo`. */
async function open(page: Page, ipc: Ipc, files: string[], saveTo: string): Promise<void> {
  const sources = await Promise.all(
    files.map(async (file) => unwrap(await ipc.call("e2e_source_info", { path: input(file) }, z.json()))),
  );
  await bridge(page, ipc, "main", {
    take_pending_encoder_sources: () => sources,
    save_encoded: async (args) => {
      const { jobId } = saveArgs.parse(args);
      return unwrap(await ipc.call("e2e_save_encoded", { jobId, dest: saveTo }, z.string()));
    },
  });
  await page.goto("/");
}

const encodeAll = (page: Page) => page.getByRole("button", { name: "Encode all" });
const save = (page: Page) => page.getByRole("button", { name: "Save…", exact: true });

test.use({ viewport: { width: 1400, height: 900 } });

test("drop an image, Encode all, Save: a valid KTX2 that the wipe shows", async ({ page, ipc }, testInfo) => {
  const saveTo = testInfo.outputPath("saved.ktx2");
  await open(page, ipc, ["small-rgba.png"], saveTo);
  await expect(encodeAll(page)).toBeEnabled();
  await encodeAll(page).click();
  await expect(save(page)).toBeEnabled({ timeout: 60_000 });
  await save(page).click();
  await expect.poll(() => existsSync(saveTo)).toBe(true);

  const validation = await ktxValidate(saveTo);
  expect(validation.valid, validation.report).toBe(true);

  // Wipe: one side is the dropped source, the other the encoded file, texel for texel.
  const images = page.locator(".viewer-frame img");
  await expect(images).toHaveCount(2);
  const sides = await Promise.all(
    [0, 1].map(async (i) => {
      const src = await images.nth(i).getAttribute("src");
      const bytes = await page.evaluate(async (url) => {
        const response = await fetch(url ?? "");
        return Array.from(new Uint8Array(await response.arrayBuffer()));
      }, src);
      return readRgba(Buffer.from(bytes));
    }),
  );
  const source = await readRgba(input("small-rgba.png"));
  const encoded = await decode(saveTo);
  const diffs = sides.map((side) => [maxAbsDiff(side, source), maxAbsDiff(side, encoded)]);
  expect(diffs.some(([toSource]) => toSource === 0), `a side shows the source: ${JSON.stringify(diffs)}`).toBe(true);
  expect(diffs.some(([, toFile]) => toFile === 0), `a side shows the saved file: ${JSON.stringify(diffs)}`).toBe(true);
});

test("packing images of different sizes keeps Encode disabled", async ({ page, ipc }, testInfo) => {
  await open(page, ipc, ["frame_01.png", "frame_02.png", "frame_big.png"], testInfo.outputPath("x.ktx2"));
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.getByRole("button", { name: /Pack as/ }).click();
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect(encodeAll(page)).toBeDisabled();
});

test("packing matching images as layers allows Encode", async ({ page, ipc }, testInfo) => {
  await open(page, ipc, ["frame_01.png", "frame_02.png", "frame_03.png", "frame_04.png"], testInfo.outputPath("x.ktx2"));
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.getByRole("button", { name: /Pack as/ }).click();
  await expect(encodeAll(page)).toBeEnabled();
});

test("View → Actual size and Fit work on the encoder preview", async ({ page, ipc }, testInfo) => {
  await open(page, ipc, ["small-rgba.png"], testInfo.outputPath("x.ktx2"));
  const img = page.locator(".viewer-frame img").first();
  await expect(img).toBeVisible();
  await menu(page, "zoom-actual");
  await expect.poll(async () => Math.round((await img.boundingBox())?.width ?? 0)).toBe(64);
  await menu(page, "zoom-fit");
  await expect.poll(async () => Math.round((await img.boundingBox())?.width ?? 0)).toBeGreaterThan(64);
});
