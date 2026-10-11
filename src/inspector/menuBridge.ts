import { on } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { getViewerControls } from "./viewerControls";

function handleMenuAction(action: MenuAction): void {
  const viewer = getViewerControls();
  switch (action) {
    case "zoom-in":
      viewer?.zoomIn();
      break;
    case "zoom-out":
      viewer?.zoomOut();
      break;
    case "zoom-actual":
      viewer?.zoomActual();
      break;
    case "zoom-fit":
      viewer?.zoomFit();
      break;
    case "encode":
    case "cancel-encode":
    case "save":
    case "clear-sources":
    case "export":
      break;
  }
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
