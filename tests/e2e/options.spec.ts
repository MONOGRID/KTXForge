/**
 * 1. The GUI cannot build a `ktx create` command line the tool rejects.
 *
 * A seeded "user" opens an output, picks a preset, then changes random controls,
 * touching only fields the rules engine shows (`enabled`) and only values the
 * control offers (`choices`, or what a user can type into the input). After every
 * edit the store's suggestion step runs. Pressing Encode must then produce a file.
 * A failure prints the seed, the edits and the options, which replay exactly.
 */
import type { InputRole } from "../../src/bindings/InputRole";
import { controlFor, oneOf, prng } from "../support/controls";
import { input } from "../support/paths";
import { EncoderSession, getPath } from "../support/session";
import { expect, test } from "../support/test";

const PRESETS = [
  null,
  "official.color.uastc-rgba",
  "official.color.basislz-rgb",
  "official.data.uastc-rgb",
  "official.normal.uastc",
  "official.normal.basis-lz",
  "official.mask.uastc-r8",
  "official.mask.basislz-r8",
];

const frames = ["frame_01.png", "frame_02.png", "frame_03.png", "frame_04.png"];
const faces = [0, 1, 2, 3, 4, 5].map((i) => `face_${String(i)}.png`);

/** What a user drops, per role. */
const OUTPUTS: { name: string; role: InputRole; files: string[] }[] = [
  { name: "RGBA PNG", role: "base2D", files: ["small-rgba.png"] },
  { name: "RGB PNG", role: "base2D", files: ["small-rgb.png"] },
  { name: "grey PNG", role: "base2D", files: ["small-gray.png"] },
  { name: "JPEG", role: "base2D", files: ["small.jpg"] },
  { name: "odd-sized PNG", role: "base2D", files: ["odd-37x19.png"] },
  { name: "Radiance HDR", role: "base2D", files: ["ramp.hdr"] },
  { name: "array layers", role: "arrayLayers", files: frames },
  { name: "cubemap", role: "cubemapFaces", files: faces },
  { name: "mip chain", role: "mipChain", files: [0, 1, 2, 3, 4, 5, 6].map((l) => `mip_${String(l)}.png`) },
  { name: "depth slices", role: "depthSlices", files: [0, 1, 2, 3].map((i) => `slice_${String(i)}.png`) },
];

const SEEDS_PER_OUTPUT = 8;
const EDITS_PER_SESSION = 6;

for (const output of OUTPUTS) {
  test.describe(`${output.name} (${output.role})`, () => {
    for (let seed = 1; seed <= SEEDS_PER_OUTPUT; seed++) {
      test(`random session #${String(seed)} encodes`, async ({ ipc }, testInfo) => {
        const random = prng(seed * 1009 + output.name.length * 31);
        const preset = oneOf(PRESETS, random);
        const session = await EncoderSession.open(
          ipc,
          output.files.map((f) => input(f)),
          output.role,
          preset,
        );

        for (let i = 0; i < EDITS_PER_SESSION; i++) {
          const report = await session.report();
          const editable = report.fields.flatMap((field) => {
            const control = controlFor(field.key);
            return field.enabled && control !== undefined ? [{ field, control }] : [];
          });
          if (editable.length === 0) break;
          const { field, control } = oneOf(editable, random);
          const value = control.pick(field, getPath(session.options, control.path), random);
          await session.set(field.key, control.path, value);
        }

        const outcome = await session.encode(testInfo.outputPath("out.ktx2"));
        expect(outcome.ok, outcome.ok ? "" : session.failure(outcome.error)).toBe(true);
      });
    }
  });
}
