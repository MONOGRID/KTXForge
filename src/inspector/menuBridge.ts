import { subscribe } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { applyZoomMenuAction } from "../shared/menuZoom";

function handleMenuAction(action: MenuAction): void {
  applyZoomMenuAction(action);
}

/** Wire native View menu zoom actions for the inspector window. */
export function bindInspectorMenu(): () => void {
  return subscribe("menu://action", (action) => {
    handleMenuAction(action);
  });
}
