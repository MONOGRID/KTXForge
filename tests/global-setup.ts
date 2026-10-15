import process from "node:process";
import { z } from "zod";
import { generateInputs } from "./support/inputs";
import { copyFile } from "node:fs/promises";
import { join } from "node:path";
import { generateCorpus } from "./support/corpus";
import { corpusDir, fixture, fixtures, inputsDir, ktxPath, manifestPath } from "./support/paths";
import { run } from "./support/run";

const artifactSchema = z.object({
  reason: z.literal("compiler-artifact"),
  target: z.object({ name: z.string() }),
  executable: z.string().nullable(),
});

/** Build the headless driver and find its path from cargo's own output. */
async function buildDriver(): Promise<string> {
  const out = await run("cargo", [
    "build",
    "--manifest-path",
    manifestPath,
    "--features",
    "e2e",
    "--bin",
    "ktxforge-e2e",
    "--message-format",
    "json",
  ]);
  if (out.code !== 0) {
    throw new Error(`cargo build of ktxforge-e2e failed:\n${out.stderr}`);
  }
  for (const line of out.stdout.split("\n")) {
    if (!line.startsWith("{")) continue;
    const parsed: unknown = JSON.parse(line);
    const artifact = artifactSchema.safeParse(parsed);
    if (artifact.success && artifact.data.target.name === "ktxforge-e2e" && artifact.data.executable) {
      return artifact.data.executable;
    }
  }
  throw new Error("cargo did not report the ktxforge-e2e executable");
}

export default async function globalSetup(): Promise<void> {
  const [driver] = await Promise.all([buildDriver(), generateInputs(inputsDir)]);
  // Workers inherit the environment of the runner process.
  process.env.KTXFORGE_E2E_DRIVER = driver;
  await copyFile(fixture("hdr.exr"), join(inputsDir, "ramp.exr"));
  const refused = await generateCorpus(ktxPath(), inputsDir, fixtures, corpusDir);
  if (refused.length > 0) {
    process.stdout.write(`ktx create refused ${String(refused.length)} corpus recipes:\n${refused.join("\n")}\n`);
  }
}
