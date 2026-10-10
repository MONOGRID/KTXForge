// fallow-ignore-file unused-file
// consumed by T5.1
/**
 * Typed event listeners. Bound to generated payloads in T2.6.
 */
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Events = {
  "encoder://sources-added": unknown;
  "encoder://drop-rejected": unknown;
  "encode://progress": unknown;
  "inspector://file": unknown;
  "update://status": unknown;
};

/** @expected-unused consumed by T5.1 */
export async function on<K extends keyof Events>(
  name: K,
  handler: (payload: Events[K]) => void,
): Promise<UnlistenFn> {
  return listen<Events[K]>(name, (event) => {
    handler(event.payload);
  });
}
