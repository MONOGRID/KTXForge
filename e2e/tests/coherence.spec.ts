/**
 * 5. What the user chooses in the GUI is what ends up in the file, and the
 * encoder preview shows that file. Goldens (decoded images and header summaries)
 * catch any drift when the bundled `ktx` changes, e.g. a move to KTX-Software 5.
 */
import { readFile } from "node:fs/promises";
import { z } from "zod";
import type { InputRole } from "../../src/bindings/InputRole";
import type { OptionKey } from "../../src/bindings/OptionKey";
import { edit } from "../support/controls";
import { bytesSchema, unwrap, type Ipc, type Json } from "../support/ipc";
import { counts, decode, isFloat, ktxInfo, ktxValidate, type KtxInfo } from "../support/ktx";
import { input } from "../support/paths";
import { maxAbsDiff, pngOf, psnr, readRgba } from "../support/pixels";
import { EncoderSession, type Encoded } from "../support/session";
import { expect, test } from "../support/test";

type TestInfo = Parameters<Parameters<typeof test>[2]>[1];

/** Encode, require a valid file, and return what Khronos' tool reads from it. */
async function encodeValid(
  session: EncoderSession,
  testInfo: TestInfo,
  name = "out",
  animation: Parameters<EncoderSession["encode"]>[1] = null,
): Promise<{ out: Encoded; info: KtxInfo }> {
  const outcome = await session.encode(testInfo.outputPath(`${name}.ktx2`), animation);
  if (!outcome.ok) throw new Error(session.failure(outcome.error));
  const validation = await ktxValidate(outcome.value.file);
  expect(validation.valid, validation.report).toBe(true);
  return { out: outcome.value, info: await ktxInfo(outcome.value.file) };
}

/** Header facts a format or tool change would show up in. The writer string is left out. */
function summary(info: KtxInfo): string {
  const block = info.dataFormatDescriptor.blocks[0];
  const keys = Object.entries(info.keyValueData).filter(([k]) => k !== "KTXwriter" && k !== "KTXwriterScParams");
  return `${JSON.stringify({ header: info.header, colorModel: block.colorModel, transfer: block.transferFunction, primaries: block.colorPrimaries, keyValue: Object.fromEntries(keys) }, null, 2)}\n`;
}

/** The encoded side of the encoder's wipe, for one image. */
async function encoderPreview(ipc: Ipc, out: Encoded): Promise<Buffer> {
  return unwrap(
    await ipc.call(
      "encoded_preview",
      { jobId: out.jobId, selection: { level: 0, face: 0, layer: 0, depthSlice: 0 } },
      bytesSchema,
    ),
  );
}

const presetSchema = z.array(
  z.object({
    id: z.string(),
    owned: z.object({
      format: z.string(),
      encode: z.enum(["uastc", "basis-lz"]).nullable(),
      normalize: z.boolean(),
      supercompression: z.object({ kind: z.enum(["none", "zstd", "zlib"]) }),
    }),
  }),
);

const CHANNELS = [0, 1, 2, 3] as const;
const MODEL = { uastc: "KHR_DF_MODEL_UASTC", "basis-lz": "KHR_DF_MODEL_ETC1S" } as const;
const SCHEME = { none: "KTX_SS_NONE", zstd: "KTX_SS_ZSTD", zlib: "KTX_SS_ZLIB" } as const;

/** The input each Official preset is meant for. */
function presetSource(id: string): string {
  return id.includes(".mask.") ? "mask.png" : id.includes(".normal.") ? "normal.png" : "color-rgba.png";
}

test("every Official preset writes the format, codec and supercompression it names", async ({ ipc }, testInfo) => {
  const catalog = unwrap(await ipc.call("list_encode_presets", {}, presetSchema));
  for (const preset of catalog) {
    await test.step(preset.id, async () => {
      const source = presetSource(preset.id);
      const session = await EncoderSession.open(ipc, [input(source)], "base2D", preset.id);
      const { out, info } = await encodeValid(session, testInfo, preset.id);
      const block = info.dataFormatDescriptor.blocks[0];
      const scheme = preset.owned.encode === "basis-lz" ? "KTX_SS_BASIS_LZ" : SCHEME[preset.owned.supercompression.kind];
      expect.soft(info.header.supercompressionScheme, "supercompression").toBe(scheme);
      if (preset.owned.encode !== null) {
        expect.soft(block.colorModel, "codec").toBe(MODEL[preset.owned.encode]);
      }
      expect.soft(block.transferFunction, "transfer").toBe(
        preset.owned.format.endsWith("_SRGB") ? "KHR_DF_TRANSFER_SRGB" : "KHR_DF_TRANSFER_LINEAR",
      );

      // The wipe shows the file, texel for texel.
      const decoded = await decode(out.file);
      expect.soft(maxAbsDiff(decoded, await readRgba(await encoderPreview(ipc, out))), "preview").toBe(0);

      expect.soft(summary(info)).toMatchSnapshot(`preset-${preset.id}.json`);
      expect.soft(await pngOf(decoded)).toMatchSnapshot(`preset-${preset.id}.png`);
    });
  }
});

test("linear Official presets store the PNG's bytes, not an sRGB → linear conversion", async ({ ipc }, testInfo) => {
  const catalog = unwrap(await ipc.call("list_encode_presets", {}, presetSchema));
  const linear = catalog.filter((p) => !p.owned.format.endsWith("_SRGB"));
  expect(linear.length).toBeGreaterThan(0);
  for (const preset of linear) {
    await test.step(preset.id, async () => {
      const source = presetSource(preset.id);
      const session = await EncoderSession.open(ipc, [input(source)], "base2D", preset.id);
      // Uncompressed, so any difference is a color conversion, not codec loss.
      await edit(session, "encode", null);
      const { out } = await encodeValid(session, testInfo, `linear-${preset.id}`);
      const channels = CHANNELS.slice(0, preset.owned.format.split("8").length - 1);
      // `--normalize` re-quantizes unit vectors by one step; a transfer conversion moves them by far more.
      expect.soft(maxAbsDiff(await readRgba(input(source)), await decode(out.file), channels)).toBeLessThanOrEqual(
        preset.owned.normalize ? 1 : 0,
      );
    });
  }
});

for (const [name, file] of [
  ["LDR", "small-rgba.png"],
  ["HDR", "ramp.hdr"],
] as const) {
  test(`every format the rail offers for ${name} sources is the file's vkFormat`, async ({ ipc }, testInfo) => {
    const uncompressed = async (session: EncoderSession): Promise<void> => {
      // HDR sources never show the codec select.
      const shown = (await session.report()).fields.some((f) => f.key === "encode" && f.enabled);
      if (shown) await edit(session, "encode", null);
    };
    const probe = await EncoderSession.open(ipc, [input(file)]);
    await uncompressed(probe);
    const offered = (await probe.report()).fields.find((f) => f.key === "format")?.choices.map((c) => c.value) ?? [];
    expect(offered.length).toBeGreaterThan(0);
    for (const format of offered) {
      await test.step(format, async () => {
        const session = await EncoderSession.open(ipc, [input(file)]);
        await uncompressed(session);
        await edit(session, "format", format);
        const { info } = await encodeValid(session, testInfo, format);
        expect.soft(info.header.vkFormat).toBe(`VK_FORMAT_${format}`);
      });
    }
  });
}

type Case = {
  name: string;
  file?: string;
  edits: [OptionKey, Json][];
  expect: (info: KtxInfo) => void;
};

const CASES: Case[] = [
  {
    name: "UASTC writes a UASTC payload",
    edits: [["encode", "uastc"], ["zstd", { kind: "none" }]],
    expect: (info) => {
      expect(info.dataFormatDescriptor.blocks[0]?.colorModel).toBe("KHR_DF_MODEL_UASTC");
      expect(info.header.supercompressionScheme).toBe("KTX_SS_NONE");
    },
  },
  {
    name: "BasisLZ writes ETC1S with BasisLZ supercompression",
    edits: [["encode", "basis-lz"]],
    expect: (info) => {
      expect(info.dataFormatDescriptor.blocks[0]?.colorModel).toBe("KHR_DF_MODEL_ETC1S");
      expect(info.header.supercompressionScheme).toBe("KTX_SS_BASIS_LZ");
    },
  },
  {
    name: "Zstd supercompresses",
    edits: [["zstd", { kind: "zstd", level: 5 }]],
    expect: (info) => { expect(info.header.supercompressionScheme).toBe("KTX_SS_ZSTD") },
  },
  {
    name: "Zlib supercompresses",
    edits: [["zlib", { kind: "zlib", level: 5 }]],
    expect: (info) => { expect(info.header.supercompressionScheme).toBe("KTX_SS_ZLIB") },
  },
  {
    name: "Generate mipmaps writes the full chain",
    edits: [["generate-mipmap", true]],
    expect: (info) => { expect(info.header.levelCount).toBe(7) },
  },
  {
    name: "Generate mipmaps with a level count writes that many",
    edits: [["generate-mipmap", true], ["levels", 3]],
    expect: (info) => { expect(info.header.levelCount).toBe(3) },
  },
  {
    name: "Scale resizes",
    edits: [["scale", 0.5]],
    expect: (info) => { expect([info.header.pixelWidth, info.header.pixelHeight]).toEqual([32, 32]) },
  },
  {
    name: "Width and height resize",
    edits: [["scale", null], ["width", 100], ["height", 50]],
    expect: (info) => { expect([info.header.pixelWidth, info.header.pixelHeight]).toEqual([100, 50]) },
  },
  {
    name: "Swizzle is recorded",
    edits: [["encode", null], ["swizzle", "bgra"]],
    expect: (info) => { expect(info.keyValueData.KTXswizzle).toBe("bgra") },
  },
  {
    name: "Assigned primaries are recorded",
    edits: [["assign-primaries", "bt2020"]],
    expect: (info) => { expect(info.dataFormatDescriptor.blocks[0]?.colorPrimaries).toBe("KHR_DF_PRIMARIES_BT2020") },
  },
  {
    name: "Assigned bottom-left origin is recorded",
    edits: [["assign-texcoord-origin", "bottom-left"]],
    expect: (info) => { expect(info.keyValueData.KTXorientation).toBe("ru") },
  },
  {
    name: "A linear format keeps a linear transfer",
    edits: [["encode", null], ["format", "R8G8B8A8_UNORM"], ["assign-tf", "linear"]],
    expect: (info) => { expect(info.dataFormatDescriptor.blocks[0]?.transferFunction).toBe("KHR_DF_TRANSFER_LINEAR") },
  },
];

for (const c of CASES) {
  test(c.name, async ({ ipc }, testInfo) => {
    const session = await EncoderSession.open(ipc, [input(c.file ?? "small-rgba.png")]);
    for (const [key, value] of c.edits) await edit(session, key, value);
    const { info } = await encodeValid(session, testInfo);
    c.expect(info);
  });
}

const frames = [1, 2, 3, 4].map((i) => `frame_0${String(i)}.png`);
const faces = [0, 1, 2, 3, 4, 5].map((i) => `face_${String(i)}.png`);
const mips = [0, 1, 2, 3, 4, 5, 6].map((l) => `mip_${String(l)}.png`);
const slices = [0, 1, 2, 3].map((i) => `slice_${String(i)}.png`);

const PACKS: { role: InputRole; files: string[]; at: (i: number) => { level?: number; face?: number; layer?: number; depthSlice?: number } }[] = [
  { role: "arrayLayers", files: frames, at: (i) => ({ layer: i }) },
  { role: "cubemapFaces", files: faces, at: (i) => ({ face: i }) },
  { role: "mipChain", files: mips, at: (i) => ({ level: i }) },
  { role: "depthSlices", files: slices, at: (i) => ({ depthSlice: i }) },
];

for (const pack of PACKS) {
  test(`${pack.role}: input n lands in image n`, async ({ ipc }, testInfo) => {
    const session = await EncoderSession.open(ipc, pack.files.map(input), pack.role);
    await edit(session, "encode", null);
    const { out, info } = await encodeValid(session, testInfo);
    expect(isFloat(info)).toBe(false);
    for (const [i, file] of pack.files.entries()) {
      const decoded = await decode(out.file, pack.at(i));
      // Uncompressed: each image is its own input, not a neighbour's.
      expect(maxAbsDiff(await readRgba(input(file)), decoded), `${file} → ${JSON.stringify(pack.at(i))}`).toBe(0);
    }
    expect(summary(info)).toMatchSnapshot(`pack-${pack.role}.json`);
  });
}

for (const preset of ["official.color.uastc-rgba", "official.color.basislz-rgba"]) {
  test(`animation metadata survives ${preset} (frames stay in order)`, async ({ ipc }, testInfo) => {
    const session = await EncoderSession.open(ipc, frames.map(input), "arrayLayers", preset);
    const animation = { duration: 3, timescale: 60, loopCount: 2 };
    const { out, info } = await encodeValid(session, testInfo, "anim", animation);
    expect(info.keyValueData.KTXanimData).toEqual(animation);
    expect(counts(info).layers).toBe(4);
    for (const [i, file] of frames.entries()) {
      const decoded = await decode(out.file, { layer: i });
      const scores = await Promise.all(frames.map(async (f) => psnr(await readRgba(input(f)), decoded)));
      // Lossy, so identity by best match: layer i looks most like frame i.
      expect(scores.indexOf(Math.max(...scores)), `layer ${String(i)} is ${file}`).toBe(i);
    }
    expect(summary(info)).toMatchSnapshot(`anim-${preset}.json`);
  });
}

test("WebP, TIFF, JPEG and awkward file names reach the file unchanged", async ({ ipc }, testInfo) => {
  const reference = await readRgba(input("color-rgba.png"));
  for (const file of ["color-rgba.webp", "color-rgba.tiff", "comma, name.png", "@at-name.png"]) {
    await test.step(file, async () => {
      const session = await EncoderSession.open(ipc, [input(file)]);
      await edit(session, "encode", null);
      const { out } = await encodeValid(session, testInfo, file.replace(/[^a-z0-9]/gi, "_"));
      expect.soft(maxAbsDiff(reference, await decode(out.file))).toBe(0);
    });
  }
  await test.step("photo.jpg", async () => {
    const session = await EncoderSession.open(ipc, [input("photo.jpg")]);
    await edit(session, "encode", null);
    const { out } = await encodeValid(session, testInfo, "photo");
    // JPEG decoders (the app's vs libjpeg-turbo here) round differently.
    expect.soft(maxAbsDiff(await readRgba(await readFile(input("photo.jpg"))), await decode(out.file))).toBeLessThanOrEqual(8);
  });
});
