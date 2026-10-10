// fallow-ignore-file unused-file
// consumed by T5.1
/**
 * Typed invoke wrappers. Bound to generated `src/bindings` types in T2.6.
 */
import { invoke } from "@tauri-apps/api/core";

export type Commands = {
  ktx_version: { args: Record<string, never>; result: unknown };
  default_create_options: { args: Record<string, never>; result: unknown };
};

/** @expected-unused consumed by T5.1 */
export async function call<K extends keyof Commands>(
  name: K,
  args: Commands[K]["args"],
): Promise<Commands[K]["result"]> {
  return invoke(name, args);
}
