// fallow-ignore-file unused-file
// consumed by T5.1
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DropRejectedPayload } from "../bindings/DropRejectedPayload";
import type { EncodeProgressPayload } from "../bindings/EncodeProgressPayload";
import type { InspectorFilePayload } from "../bindings/InspectorFilePayload";
import type { SourceInfo } from "../bindings/SourceInfo";
import type { UpdateStatusPayload } from "../bindings/UpdateStatusPayload";

export type Events = {
  "encoder://sources-added": SourceInfo[];
  "encoder://drop-rejected": DropRejectedPayload;
  "encode://progress": EncodeProgressPayload;
  "inspector://file": InspectorFilePayload;
  "update://status": UpdateStatusPayload;
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
