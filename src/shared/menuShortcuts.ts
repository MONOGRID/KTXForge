import { useEventListener } from "@vueuse/core";
import { call } from "../api/commands";

type AcceleratedMenuId =
  | "open"
  | "encode"
  | "cancel-encode"
  | "save"
  | "export"
  | "exit"
  | "zoom-in"
  | "zoom-out"
  | "zoom-actual"
  | "zoom-fit";

type ShortcutEvent = Pick<
  KeyboardEvent,
  "ctrlKey" | "metaKey" | "altKey" | "shiftKey" | "code"
>;

/**
 * Keys are `${code}:any` (shift ignored) or `${code}:0` / `${code}:1`
 * (shift required false/true). Mirrors accelerators in `menu.rs`.
 */
const SHORTCUT_LOOKUP = new Map<string, AcceleratedMenuId>([
  ["KeyO:0", "open"],
  ["Enter:0", "encode"],
  ["Period:0", "cancel-encode"],
  ["KeyS:0", "save"],
  ["KeyE:1", "export"],
  ["KeyQ:0", "exit"],
  ["Equal:any", "zoom-in"],
  ["NumpadAdd:any", "zoom-in"],
  ["Minus:any", "zoom-out"],
  ["NumpadSubtract:any", "zoom-out"],
  ["Digit1:0", "zoom-actual"],
  ["Digit0:0", "zoom-fit"],
]);

function isCtrlOnlyShortcut(event: ShortcutEvent): boolean {
  return event.ctrlKey && !event.metaKey && !event.altKey;
}

function resolveShortcutId(
  code: string,
  shiftKey: boolean,
): AcceleratedMenuId | null {
  const flexible = SHORTCUT_LOOKUP.get(`${code}:any`);
  if (flexible !== undefined) {
    return flexible;
  }
  const exact = SHORTCUT_LOOKUP.get(`${code}:${shiftKey ? "1" : "0"}`);
  return exact ?? null;
}

/**
 * Map a Ctrl-chord keydown to the menu item id declared in `menu.rs`.
 * Used as the Windows fallback when native muda accelerators do not fire
 * under WebView2 (wry#451).
 */
function matchMenuShortcut(event: ShortcutEvent): AcceleratedMenuId | null {
  if (!isCtrlOnlyShortcut(event)) {
    return null;
  }
  return resolveShortcutId(event.code, event.shiftKey);
}

function isWindowsHost(): boolean {
  return navigator.userAgent.includes("Windows");
}

/**
 * On Windows only, listen for the same Ctrl chords the native menu shows and
 * dispatch them through Rust so enablement and handlers stay shared.
 * No-op on macOS/Linux where AppKit/GTK accelerators already work.
 */
export function bindWindowsMenuShortcuts(): () => void {
  if (!isWindowsHost()) {
    return () => {};
  }
  return useEventListener(
    window,
    "keydown",
    (event: KeyboardEvent) => {
      if (event.repeat) {
        return;
      }
      const id = matchMenuShortcut(event);
      if (id === null) {
        return;
      }
      event.preventDefault();
      void call("dispatch_menu_action", { id }).catch(() => {
        // Menu may be unavailable on early boot.
      });
    },
    { capture: true },
  );
}
