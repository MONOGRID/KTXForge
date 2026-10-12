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
  ["Numpad1:0", "zoom-actual"],
  ["Digit0:0", "zoom-fit"],
  ["Numpad0:0", "zoom-fit"],
]);

function isMacHost(): boolean {
  return navigator.userAgent.includes("Mac");
}

function isWindowsHost(): boolean {
  return navigator.userAgent.includes("Windows");
}

function isZoomShortcutId(id: AcceleratedMenuId): boolean {
  return (
    id === "zoom-in" ||
    id === "zoom-out" ||
    id === "zoom-actual" ||
    id === "zoom-fit"
  );
}

/** Cmd-only on macOS, Ctrl-only elsewhere (matches CmdOrCtrl menu accelerators). */
function isPrimaryModOnlyShortcut(event: ShortcutEvent): boolean {
  if (event.altKey) {
    return false;
  }
  if (isMacHost()) {
    return event.metaKey && !event.ctrlKey;
  }
  return event.ctrlKey && !event.metaKey;
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
 * Map a primary-mod chord keydown to the menu item id declared in `menu.rs`.
 * Used when native muda accelerators do not fire (WebView2; some Linux zoom
 * chords) or as a reliable path alongside OS-focused menu clicks.
 */
export function matchMenuShortcut(
  event: ShortcutEvent,
): AcceleratedMenuId | null {
  if (!isPrimaryModOnlyShortcut(event)) {
    return null;
  }
  return resolveShortcutId(event.code, event.shiftKey);
}

/**
 * Capture-phase keydown → `dispatch_menu_action` so enablement and handlers
 * stay shared with native menu activation.
 *
 * - Windows: all File/View chords (WebView2 drops native accelerators, wry#451).
 * - Linux / macOS: View zoom chords only — File accelerators usually work via
 *   GTK/AppKit; synthesizing them too would double-fire Open/Encode/etc.
 */
export function bindMenuShortcuts(): () => void {
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
      if (!isWindowsHost() && !isZoomShortcutId(id)) {
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
