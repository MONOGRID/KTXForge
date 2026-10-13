import { getAllWindows, getCurrentWindow } from "@tauri-apps/api/window";

/**
 * OS-level focus for View-menu zoom routing across encoder/inspector.
 *
 * Native menus (especially GTK on Linux) steal focus while the menu is open:
 * `document.hasFocus()` is false, and Tauri often reports the window as
 * unfocused too. Keyboard shortcuts still see a focused window, which is why
 * they worked when menu clicks did not.
 *
 * Eligibility sticks on blur unless another of our windows is focused.
 */
let eligibleForMenuZoom = true;
/** Bumped on focus-gain so a stale blur async cannot overwrite a later focus. */
let focusGeneration = 0;

/**
 * Pure eligibility update for tests and the focus listener.
 * Focus gained → eligible. Blur → keep eligibility unless another app window
 * took focus (so GTK menu blur does not drop View → Zoom).
 */
export function nextMenuZoomEligibility(
  currentlyEligible: boolean,
  focused: boolean,
  otherAppWindowFocused: boolean,
): boolean {
  if (focused) {
    return true;
  }
  if (otherAppWindowFocused) {
    return false;
  }
  return currentlyEligible;
}

async function isOtherAppWindowFocused(selfLabel: string): Promise<boolean> {
  const windows = await getAllWindows();
  const flags = await Promise.all(
    windows.map(async (window) => {
      if (window.label === selfLabel) {
        return false;
      }
      return window.isFocused();
    }),
  );
  return flags.some(Boolean);
}

/**
 * Track Tauri window focus for View-menu zoom routing across encoder/inspector.
 */
export function bindMenuZoomFocusGate(): () => void {
  const win = getCurrentWindow();
  void win.isFocused().then((focused) => {
    eligibleForMenuZoom = focused;
  });
  const unlistenPromise = win.onFocusChanged((event) => {
    const focused = event.payload;
    if (focused) {
      focusGeneration += 1;
      eligibleForMenuZoom = true;
      return;
    }
    const generation = focusGeneration;
    void isOtherAppWindowFocused(win.label).then((otherFocused) => {
      if (generation !== focusGeneration) {
        return;
      }
      eligibleForMenuZoom = nextMenuZoomEligibility(
        eligibleForMenuZoom,
        false,
        otherFocused,
      );
    });
  });
  return () => {
    void unlistenPromise.then((unlisten) => {
      unlisten();
    });
  };
}

/** True when this webview should apply app-wide View zoom menu actions. */
export function isMenuZoomWindowFocused(): boolean {
  return eligibleForMenuZoom;
}
