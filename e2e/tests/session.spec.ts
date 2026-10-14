/**
 * Session behaviour around encodes: the bundled tool, inputs the sidecar used to
 * crash on, Cancel, metrics, and job ids coming from the webview.
 */
import { existsSync } from "node:fs";
import { z } from "zod";
import { unwrap } from "../support/ipc";
import { ktxInfo, ktxValidate } from "../support/ktx";
import { fixture, input } from "../support/paths";
import { EncoderSession } from "../support/session";
import { edit } from "../support/controls";
import { expect, test } from "../support/test";

test("the bundled ktx is the pinned KTX-Software version", async ({ ipc }) => {
  const version = unwrap(await ipc.call("ktx_version", {}, z.object({ pinned: z.string(), reported: z.string() })));
  expect(version.reported).toContain(`v${version.pinned}`);
});

test("a tiled EXR encodes (the Linux ktx crashes on tiled input)", async ({ ipc }, testInfo) => {
  const session = await EncoderSession.open(ipc, [fixture("tiled.exr")]);
  const out = unwrap(await session.encode(testInfo.outputPath("tiled.ktx2")));
  expect((await ktxValidate(out.file)).valid).toBe(true);
  const info = await ktxInfo(out.file);
  expect(info.header.vkFormat).toBe("VK_FORMAT_R16G16B16A16_SFLOAT");
  expect([info.header.pixelWidth, info.header.pixelHeight]).toEqual([32, 32]);
});

test("SSIM and PSNR are reported when asked for", async ({ ipc }, testInfo) => {
  const session = await EncoderSession.open(ipc, [input("small-rgba.png")]);
  await edit(session, "compare-ssim", true);
  await edit(session, "compare-psnr", true);
  const out = unwrap(await session.encode(testInfo.outputPath("metrics.ktx2")));
  expect(out.ssim).toEqual(expect.any(Number));
  expect(out.psnr).toEqual(expect.any(Number));
});

test("Cancel stops a running encode and leaves nothing to save", async ({ ipc }, testInfo) => {
  const session = await EncoderSession.open(ipc, [input("big-rgba.png")]);
  await edit(session, "uastc-quality", 4);
  await edit(session, "generate-mipmap", true);
  const jobId = crypto.randomUUID();
  const dest = testInfo.outputPath("cancelled.ktx2");
  const running = session.encodeUnchecked(dest, null, jobId);
  // Wait until the job exists, then cancel it.
  await expect.poll(async () => (await ipc.invoke("cancel_encode", { jobId })).ok, { timeout: 10_000 }).toBe(true);
  const outcome = await running;
  expect(outcome.ok ? "finished" : outcome.error.kind).toBe("cancelled");
  expect(existsSync(dest)).toBe(false);
});

test("job ids that are not UUIDs never reach the filesystem", async ({ ipc }, testInfo) => {
  const session = await EncoderSession.open(ipc, [input("small-rgba.png")]);
  for (const jobId of ["../../escape", "/tmp/escape", "not-a-uuid"]) {
    const outcome = await session.encodeUnchecked(testInfo.outputPath("x.ktx2"), null, jobId);
    expect(outcome.ok, jobId).toBe(false);
    const preview = await ipc.invoke("encoded_preview", {
      jobId,
      selection: { level: 0, face: 0, layer: 0, depthSlice: 0 },
    });
    expect(preview.ok, jobId).toBe(false);
  }
});
