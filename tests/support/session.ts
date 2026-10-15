/**
 * An encoder session driven the way the webview drives it: the same IPC calls
 * in the same order, plus the store's own step of applying every rules
 * suggestion (`applySuggestion` in `src/encoder/store.ts`).
 */
import { randomUUID } from "node:crypto";
import { z } from "zod";
import type { AnimationOptions } from "../../src/bindings/AnimationOptions";
import type { AppError } from "../../src/bindings/AppError";
import type { InputRole } from "../../src/bindings/InputRole";
import { unwrap, type Ipc, type Json, type Outcome } from "./ipc";

export type JsonObject = { [key: string]: Json };

const jsonObject = z.record(z.string(), z.json());

const sourceInfoSchema = z.object({
  path: z.string(),
  kind: z.enum(["ldr", "hdr"]),
  width: z.number(),
  height: z.number(),
});

const fieldSchema = z.object({
  key: z.string(),
  enabled: z.boolean(),
  choices: z.array(z.object({ value: z.string() })),
});

const reportSchema = z.object({
  fields: z.array(fieldSchema),
  suggestion: z
    .object({
      format: z.string(),
      encode: z.string().nullable(),
      convertTf: z.string().nullable(),
      assignTf: z.string().nullable(),
    })
    .nullable(),
});

export type Field = z.infer<typeof fieldSchema>;
export type Report = z.infer<typeof reportSchema>;

const packSchema = z.object({
  panel: z.string().nullable(),
  inputs: z.array(z.object({ index: z.number(), message: z.string() })),
});

const encodeResultSchema = z.object({
  jobId: z.string(),
  outputBytes: z.number(),
  width: z.number(),
  height: z.number(),
  levels: z.number(),
  layers: z.number(),
  faces: z.number(),
  codec: z.string(),
  ssim: z.number().nullable(),
  psnr: z.number().nullable(),
  warnings: z.array(z.string()),
});

export type Encoded = z.infer<typeof encodeResultSchema> & { file: string };

function isObject(value: Json | undefined): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Write `value` at `path`, creating objects on the way. */
function setPath(target: JsonObject, path: readonly string[], value: Json): void {
  if (path.length === 0) return;
  const [head = "", ...rest] = path;
  if (rest.length === 0) {
    target[head] = value;
    return;
  }
  const next = target[head];
  const child: JsonObject = isObject(next) ? next : {};
  target[head] = child;
  setPath(child, rest, value);
}

export function getPath(target: JsonObject, path: readonly string[]): Json | undefined {
  let current: Json | undefined = target;
  for (const key of path) {
    if (!isObject(current)) return undefined;
    current = current[key];
  }
  return current;
}

export class EncoderSession {
  /** Every edit made, for a copy-pasteable failure report. */
  readonly edits: string[] = [];

  private constructor(
    private readonly ipc: Ipc,
    readonly sources: JsonObject[],
    readonly role: InputRole,
    public options: JsonObject,
  ) {}

  /**
   * Drop `paths` onto one output with `role`, on a fresh form; `preset` is the
   * Official preset picked in the header (the product default when omitted).
   */
  static async open(
    ipc: Ipc,
    paths: readonly string[],
    role: InputRole = "base2D",
    preset: string | null = null,
  ): Promise<EncoderSession> {
    const sources = await Promise.all(
      paths.map(async (path) => unwrap(await ipc.call("e2e_source_info", { path }, jsonObject))),
    );
    let options = unwrap(await ipc.call("default_create_options", {}, jsonObject));
    if (preset !== null) {
      options = unwrap(await ipc.call("apply_encode_preset", { options, id: preset }, jsonObject));
    }
    const session = new EncoderSession(ipc, sources, role, options);
    if (preset !== null) session.edits.push(`preset ${preset}`);
    await session.settle();
    return session;
  }

  /** Change one control (`key` is its `OptionKey`) in the options rail, then let the store react. */
  async set(key: string, path: readonly string[], value: Json): Promise<void> {
    setPath(this.options, path, value);
    this.edits.push(`${key} = ${JSON.stringify(value)}`);
    await this.settle();
  }

  report(): Promise<Report> {
    return this.ipc
      .call("describe_options", { options: this.options, sources: this.sources, inputRole: this.role }, reportSchema)
      .then(unwrap);
  }

  /** The store applies each suggestion and asks again until there is none. */
  async settle(): Promise<Report> {
    for (let round = 0; round < 8; round++) {
      const report = await this.report();
      const s = report.suggestion;
      if (s === null) return report;
      this.options.format = s.format;
      this.options.encode = s.encode;
      setPath(this.options, ["color", "convertTf"], s.convertTf);
      setPath(this.options, ["color", "assignTf"], s.assignTf);
    }
    throw new Error(`rules suggestion never settled:\n${this.describe()}`);
  }

  packReport(): Promise<z.infer<typeof packSchema>> {
    return this.ipc
      .call("validate_pack_inputs", { sources: this.sources, inputRole: this.role }, packSchema)
      .then(unwrap);
  }

  /** Press Encode without the UI gate (the backstop path). */
  async encodeUnchecked(
    dest: string,
    animation: AnimationOptions | null = null,
    jobId: string = randomUUID(),
  ): Promise<Outcome<Encoded>> {
    const outcome = await this.ipc.call(
      "encode",
      {
        request: {
          jobId,
          outputId: randomUUID(),
          sources: this.sources.map((s) => sourceInfoSchema.parse(s).path),
          options: this.options,
          animation,
          inputRole: this.role,
        },
      },
      encodeResultSchema,
    );
    if (!outcome.ok) return outcome;
    unwrap(await this.ipc.call("e2e_save_encoded", { jobId: outcome.value.jobId, dest }, z.string()));
    return { ok: true, value: { ...outcome.value, file: dest } };
  }

  /** Press Encode, then Save to `dest`. Encode is disabled while the Inputs panel shows errors. */
  async encode(dest: string, animation: AnimationOptions | null = null): Promise<Outcome<Encoded>> {
    const pack = await this.packReport();
    if (pack.panel !== null || pack.inputs.length > 0) {
      return {
        ok: false,
        error: { kind: "invalidInput", message: `Encode disabled: ${JSON.stringify(pack)}` },
      };
    }
    return this.encodeUnchecked(dest, animation);
  }

  /** Why an encode failed, with everything needed to replay it. */
  failure(error: AppError): string {
    return [`${error.kind}: ${error.message}`, error.kind === "ktx" ? error.stderr : "", this.describe()].join("\n");
  }

  describe(): string {
    return [
      `role: ${this.role}`,
      `sources: ${this.sources.map((s) => sourceInfoSchema.parse(s).path).join(", ")}`,
      `edits:\n  ${this.edits.join("\n  ")}`,
      `options: ${JSON.stringify(this.options)}`,
    ].join("\n");
  }
}
