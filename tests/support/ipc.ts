/**
 * Client for `ktxforge-e2e`, the app's Rust commands served over stdin/stdout.
 * Command names and argument keys are the webview's (`src/api/commands.ts`).
 */
import { spawn, type ChildProcessByStdio } from "node:child_process";
import type { Readable, Writable } from "node:stream";
import { createInterface } from "node:readline";
import { z } from "zod";
import { toAppError } from "../../src/api/guards";
import type { Commands } from "../../src/api/commands";
import type { AppError } from "../../src/bindings/AppError";
import type { InputRole } from "../../src/bindings/InputRole";

export type Outcome<T> = { ok: true; value: T } | { ok: false; error: AppError };

export type Json = z.core.util.JSONType;

/**
 * Commands as the webview calls them, except that options and sources travel as
 * JSON: tests edit them generically (by option key) and Rust's serde is the type
 * check on the way in.
 */
type Session = {
  default_create_options: { args: Record<string, never> };
  list_encode_presets: { args: Record<string, never> };
  apply_encode_preset: { args: { options: Json; id: string } };
  describe_options: { args: { options: Json; sources: Json[]; inputRole: InputRole } };
  validate_pack_inputs: { args: { sources: Json[]; inputRole: InputRole } };
  check_output_compatible: { args: { anchor: Json; candidate: Json } };
  encode: {
    args: {
      request: Omit<Commands["encode"]["args"]["request"], "options"> & { options: Json };
    };
  };
};

/** Commands the driver serves only for tests: dialogs answered with a path. */
type TestOnly = {
  e2e_source_info: { args: { path: string } };
  e2e_save_encoded: { args: { jobId: string; dest: string } };
  e2e_export_images: { args: Commands["export_images"]["args"]["request"] & { dest: string } };
};

type Direct = Pick<
  Commands,
  "encoded_preview" | "cancel_encode" | "discard_outputs" | "ktx_version" | "inspect" | "inspector_image" | "source_preview" | "source_thumbnail"
>;

export type Served = { [K in keyof Direct]: { args: Direct[K]["args"] } } & Session & TestOnly;

const responseSchema = z.union([
  z.object({ id: z.number(), ok: z.unknown() }),
  z.object({ id: z.number().nullable(), err: z.unknown() }),
]);

/** `tauri::ipc::Response` bytes. */
export const bytesSchema = z
  .object({ bytes: z.string() })
  .transform((value) => Buffer.from(value.bytes, "base64"));

export class Ipc {
  private readonly child: ChildProcessByStdio<Writable, Readable, null>;
  private readonly pending = new Map<number, (response: z.infer<typeof responseSchema>) => void>();
  private nextId = 1;

  constructor(driver: string) {
    if (driver === "") throw new Error("ktxforge-e2e is not built; run the suite through `npm run e2e`");
    // The driver logs each `ktx` command line on stderr; failures carry their own stderr.
    this.child = spawn(driver, [], { stdio: ["pipe", "pipe", "ignore"] });
    createInterface({ input: this.child.stdout }).on("line", (line) => {
      const parsed: unknown = JSON.parse(line);
      const response = responseSchema.parse(parsed);
      if (response.id === null) throw new Error(`driver rejected a request: ${line}`);
      this.pending.get(response.id)?.(response);
      this.pending.delete(response.id);
    });
  }

  /** Raw IPC: the value is whatever the command serializes. */
  invoke<K extends keyof Served>(cmd: K, args: Served[K]["args"]): Promise<Outcome<unknown>> {
    return this.invokeAny(cmd, args);
  }

  /** Any command by name, as the webview sends it (the GUI bridge). */
  invokeAny(cmd: string, args: unknown): Promise<Outcome<unknown>> {
    const id = this.nextId++;
    return new Promise((resolve) => {
      this.pending.set(id, (response) => {
        resolve("ok" in response ? { ok: true, value: response.ok } : { ok: false, error: toAppError(response.err) });
      });
      this.child.stdin.write(`${JSON.stringify({ id, cmd, args })}\n`);
    });
  }

  /** IPC with the result checked against `schema`. */
  async call<K extends keyof Served, T>(
    cmd: K,
    args: Served[K]["args"],
    schema: z.ZodType<T>,
  ): Promise<Outcome<T>> {
    const outcome = await this.invoke(cmd, args);
    return outcome.ok ? { ok: true, value: schema.parse(outcome.value) } : outcome;
  }

  close(): Promise<void> {
    return new Promise((resolve) => {
      this.child.on("close", () => {
        resolve();
      });
      this.child.stdin.end();
    });
  }
}

/** The value, or a thrown error carrying what the user would have seen. */
export function unwrap<T>(outcome: Outcome<T>): T {
  if (outcome.ok) return outcome.value;
  const { error } = outcome;
  const detail = error.kind === "ktx" ? `\n${error.stderr}` : "";
  throw new Error(`${error.kind}: ${error.message}${detail}`);
}
