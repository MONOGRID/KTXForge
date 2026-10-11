import { watchEffect } from "vue";
import { call } from "../api/commands";
import { on } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { getViewerControls } from "../shared/viewerControls";
import {
  canCancelEncode,
  canClearSources,
  canEncode,
  canSaveEncoded,
  cancelEncode,
  clearSources,
  encode,
  saveEncoded,
} from "./actions";

async function setEnabled(id: string, enabled: boolean): Promise<void> {
  try {
    await call("set_menu_item_enabled", { id, enabled });
  } catch {
    // Menu may be unavailable on some platforms / early boot.
  }
}

async function handleMenuAction(action: MenuAction): Promise<void> {
  switch (action) {
    case "encode":
      await encode();
      break;
    case "cancel-encode":
      await cancelEncode();
      break;
    case "save":
      await saveEncoded();
      break;
    case "clear-sources":
      clearSources();
      break;
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
    case "export":
      break;
  }
}

/** Wire native File menu actions and enablement for the encoder window. */
export function bindEncoderMenu(): () => void {
  const unlisteners: Array<() => void> = [];
  let disposed = false;

  void on("menu://action", (action) => {
    void handleMenuAction(action);
  }).then((unlisten) => {
    if (disposed) {
      unlisten();
      return;
    }
    unlisteners.push(unlisten);
  });

  const stopWatch = watchEffect(() => {
    void setEnabled("encode", canEncode());
    void setEnabled("cancel-encode", canCancelEncode());
    void setEnabled("save", canSaveEncoded());
    void setEnabled("clear-sources", canClearSources());
  });

  return () => {
    disposed = true;
    stopWatch();
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}
