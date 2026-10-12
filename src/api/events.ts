import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DropRejectedPayload } from "../bindings/DropRejectedPayload";
import type { EncodeProgressPayload } from "../bindings/EncodeProgressPayload";
import type { InspectorFilePayload } from "../bindings/InspectorFilePayload";
import type { MenuAction } from "../bindings/MenuAction";
import type { SourceInfo } from "../bindings/SourceInfo";
import type { UpdateStatusPayload } from "../bindings/UpdateStatusPayload";

export type Events = {
  "encoder://sources-added": SourceInfo[];
  "encoder://drop-rejected": DropRejectedPayload;
  "encode://progress": EncodeProgressPayload;
  "inspector://file": InspectorFilePayload;
  "update://status": UpdateStatusPayload;
  "menu://action": MenuAction;
};

export async function on<K extends keyof Events>(
  name: K,
  handler: (payload: Events[K]) => void,
): Promise<UnlistenFn> {
  return listen<Events[K]>(name, (event) => {
    handler(event.payload);
  });
}

/**
 * Subscribe and return a dispose that is safe if the component unmounts
 * before `listen` resolves (avoids leaking the Tauri listener).
 */
export function subscribe<K extends keyof Events>(
  name: K,
  handler: (payload: Events[K]) => void,
): () => void {
  let disposed = false;
  let unlisten: UnlistenFn | undefined;
  void on(name, handler).then((fn) => {
    if (disposed) {
      fn();
      return;
    }
    unlisten = fn;
  });
  return () => {
    disposed = true;
    unlisten?.();
  };
}
