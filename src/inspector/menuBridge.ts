import { on } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { applyZoomMenuAction } from "../shared/menuZoom";

function handleMenuAction(action: MenuAction): void {
  applyZoomMenuAction(action);
}

/** Wire native View menu zoom actions for the inspector window. */
export function bindInspectorMenu(): () => void {
  const unlisteners: Array<() => void> = [];
  let disposed = false;

  void on("menu://action", (action) => {
    handleMenuAction(action);
  }).then((unlisten) => {
    if (disposed) {
      unlisten();
      return;
    }
    unlisteners.push(unlisten);
  });

  return () => {
    disposed = true;
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}
