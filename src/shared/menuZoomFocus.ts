import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * OS-level focus for this webview. Native menus (especially GTK on Linux)
 * steal *document* focus while the window stays active — `document.hasFocus()`
 * is false during View → Zoom clicks and must not gate menu zoom.
 */
let windowFocused = true;

/**
 * Track Tauri window focus for View-menu zoom routing across encoder/inspector.
 */
export function bindMenuZoomFocusGate(): () => void {
  const win = getCurrentWindow();
  void win.isFocused().then((focused) => {
    windowFocused = focused;
  });
  const unlistenPromise = win.onFocusChanged((event) => {
    windowFocused = event.payload;
  });
  return () => {
    void unlistenPromise.then((unlisten) => {
      unlisten();
    });
  };
}

/** True when this webview's window is the OS-focused one. */
export function isMenuZoomWindowFocused(): boolean {
  return windowFocused;
}
