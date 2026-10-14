/**
 * 3. The inspector reads many different KTX2 and KTX files, and
 * 5. what it shows and exports is what Khronos' `ktx extract` gets from the file.
 *
 * The corpus is written by the official `ktx create` (formats, codecs, shapes),
 * plus checked-in fixtures (KTX1, animation) and broken files.
 */
import { readFile } from "node:fs/promises";
import { z } from "zod";
import type { ImageSelection } from "../../src/bindings/ImageSelection";
import { BROKEN_FIXTURES, CORPUS } from "../support/corpus";
import { bytesSchema, unwrap, type Ipc } from "../support/ipc";
import { counts, decode, isFloat, ktxInfo, type KtxInfo } from "../support/ktx";
import { corpus, fixture } from "../support/paths";
import { maxAbsDiff, pngOf, readRgba } from "../support/pixels";
import { expect, test } from "../support/test";

const reportSchema = z.object({
  kind: z.discriminatedUnion("kind", [
    z.object({
      kind: z.literal("ktx2"),
      info: z.object({
        valid: z.boolean(),
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
      }),
    }),
    z.object({
      kind: z.literal("ktx1"),
      header: z.object({ pixelWidth: z.number(), pixelHeight: z.number() }),
    }),
  ]),
});

function full(selection: Partial<ImageSelection>): ImageSelection {
  return { level: 0, face: 0, layer: 0, depthSlice: 0, ...selection };
}

/** First image, then the last of every dimension the file has. */
function probes(info: KtxInfo): ImageSelection[] {
  const c = counts(info);
  const all = [
    full({}),
    full({ level: c.levels - 1 }),
    full({ face: c.faces - 1 }),
    full({ layer: c.layers - 1 }),
    full({ depthSlice: c.depth - 1 }),
  ];
  return all.filter((s, i) => all.findIndex((t) => JSON.stringify(t) === JSON.stringify(s)) === i);
}

async function preview(ipc: Ipc, path: string, selection: ImageSelection): Promise<Buffer> {
  return unwrap(await ipc.call("inspector_image", { path, selection }, bytesSchema));
}

for (const name of CORPUS) {
  test(`reads ${name}`, async ({ ipc }, testInfo) => {
    const path = corpus(name);
    const report = unwrap(await ipc.call("inspect", { path }, reportSchema));

    if (report.kind.kind === "ktx1") {
      // KTX1 goes through the bundled `ktx2ktx2`; Khronos' `ktx` reads only KTX2.
      const shown = await readRgba(await preview(ipc, path, full({})));
      expect([shown.width, shown.height]).toEqual([report.kind.header.pixelWidth, report.kind.header.pixelHeight]);
      expect(await pngOf(shown)).toMatchSnapshot(`${name}.png`);
      return;
    }

    const info = await ktxInfo(path);
    // The report panels show the header Khronos' tool reads.
    expect(report.kind.info.header).toEqual(info.header);
    expect(report.kind.info.valid).toBe(info.valid);

    for (const selection of probes(info)) {
      const label = JSON.stringify(selection);
      const shown = await readRgba(await preview(ipc, path, selection));
      expect([shown.width, shown.height], `size of ${label}`).toEqual([
        Math.max(1, info.header.pixelWidth >> selection.level),
        Math.max(1, Math.max(1, info.header.pixelHeight) >> selection.level),
      ]);
      if (!isFloat(info)) {
        // LDR: the viewport shows exactly the extracted texels.
        const extracted = await decode(path, selection);
        expect(maxAbsDiff(shown, extracted), `pixels of ${label}`).toBe(0);
      }
      if (selection.level === 0 && selection.face === 0 && selection.layer === 0 && selection.depthSlice === 0) {
        expect(await pngOf(shown)).toMatchSnapshot(`${name}.png`);
      }
    }

    // Export of every image: one file per level × face × layer × slice.
    const c = counts(info);
    const exported = unwrap(
      await ipc.call(
        "e2e_export_images",
        { path, selection: { kind: "all" }, format: isFloat(info) ? { kind: "exr" } : { kind: "png" }, dest: testInfo.outputPath("export") },
        z.object({ files: z.array(z.string()) }),
      ),
    );
    expect(exported.files).toHaveLength(c.levels * c.faces * c.layers * c.depth);
    for (const file of exported.files) {
      const head = (await readFile(file)).subarray(0, 4);
      // PNG signature or OpenEXR magic.
      expect(isFloat(info) ? head.readUInt32LE(0) : head.readUInt32BE(0)).toBe(isFloat(info) ? 20000630 : 0x89504e47);
    }
  });
}

for (const name of BROKEN_FIXTURES) {
  test(`rejects ${name} cleanly`, async ({ ipc }) => {
    const path = fixture(name);
    const report = await ipc.call("inspect", { path }, reportSchema);
    // Either a report that says invalid, or an error the user can read.
    if (report.ok && report.value.kind.kind === "ktx2") {
      expect(report.value.kind.info.valid).toBe(false);
    }
    const shown = await ipc.invoke("inspector_image", { path, selection: full({}) });
    expect(shown.ok).toBe(false);
    // The app keeps working afterwards.
    expect((await ipc.call("inspect", { path: corpus("r8g8b8a8_srgb.ktx2") }, reportSchema)).ok).toBe(true);
  });
}
