import { on } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { getViewerControls } from "../shared/viewerControls";

function handleMenuAction(action: MenuAction): void {
  switch (action) {
    case "zoom-in":
    case "zoom-out":
    case "zoom-actual":
    case "zoom-fit": {
      if (!document.hasFocus()) {
        break;
      }
      const viewer = getViewerControls();
      if (action === "zoom-in") {
        viewer?.zoomIn();
      } else if (action === "zoom-out") {
        viewer?.zoomOut();
      } else if (action === "zoom-actual") {
        viewer?.zoomActual();
      } else {
        viewer?.zoomFit();
      }
      break;
    }
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
