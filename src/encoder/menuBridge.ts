import { watchEffect } from "vue";
import { call } from "../api/commands";
import { subscribe } from "../api/events";
import type { MenuAction } from "../bindings/MenuAction";
import { applyZoomMenuAction } from "../shared/menuZoom";
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

function runEncoderMenuAction(action: MenuAction): Promise<void> | null {
  if (action === "encode") {
    return encode();
  }
  if (action === "cancel-encode") {
    return cancelEncode();
  }
  if (action === "save") {
    return saveEncoded();
  }
  if (action === "clear-sources") {
    clearSources();
    return Promise.resolve();
  }
  return null;
}

async function handleMenuAction(action: MenuAction): Promise<void> {
  if (applyZoomMenuAction(action)) {
    return;
  }
  const pending = runEncoderMenuAction(action);
  if (pending !== null) {
    await pending;
  }
}

/** Wire native File menu actions and enablement for the encoder window. */
export function bindEncoderMenu(): () => void {
  const stopMenu = subscribe("menu://action", (action) => {
    void handleMenuAction(action);
  });

  const stopWatch = watchEffect(() => {
    void setEnabled("encode", canEncode());
    void setEnabled("cancel-encode", canCancelEncode());
    void setEnabled("save", canSaveEncoded());
    void setEnabled("clear-sources", canClearSources());
  });

  return () => {
    stopMenu();
    stopWatch();
  };
}
