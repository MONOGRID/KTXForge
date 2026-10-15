/**
 * 2. A wrong set of inputs for one KTX output cannot reach `ktx create`.
 *
 * Valid packs encode into the shape their role promises. Invalid packs show an
 * error in the Inputs panel (Encode disabled), and even if Encode were pressed the
 * backstop refuses them before the sidecar runs.
 */
import { z } from "zod";
import type { InputRole } from "../../src/bindings/InputRole";
import { unwrap } from "../support/ipc";
import { counts, ktxInfo, ktxValidate } from "../support/ktx";
import { input } from "../support/paths";
import { EncoderSession } from "../support/session";
import { expect, test } from "../support/test";

const frames = [1, 2, 3, 4].map((i) => `frame_0${String(i)}.png`);
const faces = [0, 1, 2, 3, 4, 5].map((i) => `face_${String(i)}.png`);
const mips = [0, 1, 2, 3, 4, 5, 6].map((l) => `mip_${String(l)}.png`);
const slices = [0, 1, 2, 3].map((i) => `slice_${String(i)}.png`);

type Shape = { levels: number; faces: number; layers: number; depth: number };

const VALID: { name: string; role: InputRole; files: string[]; shape: Shape }[] = [
  { name: "one image", role: "base2D", files: ["small-rgba.png"], shape: { levels: 1, faces: 1, layers: 1, depth: 1 } },
  { name: "four layers", role: "arrayLayers", files: frames, shape: { levels: 1, faces: 1, layers: 4, depth: 1 } },
  { name: "six faces", role: "cubemapFaces", files: faces, shape: { levels: 1, faces: 6, layers: 1, depth: 1 } },
  { name: "full mip chain", role: "mipChain", files: mips, shape: { levels: 7, faces: 1, layers: 1, depth: 1 } },
  { name: "partial mip chain", role: "mipChain", files: mips.slice(0, 3), shape: { levels: 3, faces: 1, layers: 1, depth: 1 } },
  { name: "four slices", role: "depthSlices", files: slices, shape: { levels: 1, faces: 1, layers: 1, depth: 4 } },
];

const INVALID: { name: string; role: InputRole; files: string[] }[] = [
  { name: "two images for one 2D texture", role: "base2D", files: ["small-rgba.png", "small-rgb.png"] },
  { name: "layers of different sizes", role: "arrayLayers", files: [...frames, "frame_big.png"] },
  { name: "LDR and HDR layers", role: "arrayLayers", files: [...frames.slice(0, 2), "ramp.hdr"] },
  { name: "five cubemap faces", role: "cubemapFaces", files: faces.slice(0, 5) },
  { name: "seven cubemap faces", role: "cubemapFaces", files: [...faces, "face_0.png"] },
  { name: "non-square cubemap faces", role: "cubemapFaces", files: Array.from({ length: 6 }, () => "odd-37x19.png") },
  { name: "cubemap faces of different sizes", role: "cubemapFaces", files: [...faces.slice(0, 5), "frame_big.png"] },
  { name: "mip chain out of order", role: "mipChain", files: [mips[0] ?? "", mips[2] ?? "", mips[1] ?? ""] },
  { name: "mip chain with a gap", role: "mipChain", files: [mips[0] ?? "", mips[1] ?? "", mips[3] ?? ""] },
  { name: "mip chain longer than the base allows", role: "mipChain", files: [...mips, "mip_6.png"] },
  { name: "depth slices of different sizes", role: "depthSlices", files: [...slices, "frame_big.png"] },
];

for (const pack of VALID) {
  test(`${pack.role}: ${pack.name} encodes to that shape`, async ({ ipc }, testInfo) => {
    const session = await EncoderSession.open(ipc, pack.files.map(input), pack.role);
    const pack_ = await session.packReport();
    expect(pack_).toEqual({ panel: null, inputs: [] });
    const out = unwrap(await session.encode(testInfo.outputPath("out.ktx2")));
    const validation = await ktxValidate(out.file);
    expect(validation.valid, validation.report).toBe(true);
    expect(counts(await ktxInfo(out.file))).toEqual(pack.shape);
  });
}

for (const pack of INVALID) {
  test(`${pack.role}: ${pack.name} is refused before ktx`, async ({ ipc }, testInfo) => {
    const session = await EncoderSession.open(ipc, pack.files.map(input), pack.role);
    const report = await session.packReport();
    expect(report.panel !== null || report.inputs.length > 0, "Inputs panel shows an error").toBe(true);

    const pressed = await session.encodeUnchecked(testInfo.outputPath("out.ktx2"));
    expect(pressed.ok).toBe(false);
    if (!pressed.ok) expect(pressed.error.kind, pressed.error.message).toBe("invalidInput");
  });
}

test("animation is refused unless the inputs are array layers", async ({ ipc }, testInfo) => {
  const session = await EncoderSession.open(ipc, [input("small-rgba.png")], "base2D");
  const pressed = await session.encodeUnchecked(testInfo.outputPath("out.ktx2"), {
    duration: 1,
    timescale: 30,
    loopCount: 0,
  });
  expect(pressed.ok).toBe(false);
  if (!pressed.ok) expect(pressed.error.kind).toBe("invalidInput");
});

test("an HDR output cannot join an LDR queue, and the other way round", async ({ ipc }) => {
  const info = async (file: string): Promise<z.core.util.JSONType> =>
    unwrap(await ipc.call("e2e_source_info", { path: input(file) }, z.json()));
  const ldr = await info("small-rgba.png");
  const hdr = await info("ramp.hdr");
  expect((await ipc.invoke("check_output_compatible", { anchor: ldr, candidate: hdr })).ok).toBe(false);
  expect((await ipc.invoke("check_output_compatible", { anchor: hdr, candidate: ldr })).ok).toBe(false);
  expect((await ipc.invoke("check_output_compatible", { anchor: ldr, candidate: ldr })).ok).toBe(true);
});
