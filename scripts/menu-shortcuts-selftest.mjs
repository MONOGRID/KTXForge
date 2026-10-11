/**
 * Red/green loop for Windows menu-accelerator fallback.
 * Run: node scripts/menu-shortcuts-selftest.mjs
 *
 * Native muda accelerators do not fire under WebView2 on Windows
 * (tauri-apps/wry#451). This app must keep a JS keydown fallback in sync
 * with the accelerators declared in src-tauri/src/menu.rs.
 *
 * Expected ids mirror `SHORTCUT_LOOKUP` in src/shared/menuShortcuts.ts.
 */
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

function read(rel) {
  return readFileSync(join(root, rel), "utf8");
}

function assert(cond, msg) {
  if (!cond) {
    throw new Error(msg);
  }
}

const shortcutTs = read("src/shared/menuShortcuts.ts");
const requiredEntries = [
  '["KeyO:0", "open"]',
  '["Enter:0", "encode"]',
  '["Period:0", "cancel-encode"]',
  '["KeyS:0", "save"]',
  '["KeyE:1", "export"]',
  '["KeyQ:0", "exit"]',
  '["Equal:any", "zoom-in"]',
  '["NumpadAdd:any", "zoom-in"]',
  '["Minus:any", "zoom-out"]',
  '["NumpadSubtract:any", "zoom-out"]',
  '["Digit1:0", "zoom-actual"]',
  '["Digit0:0", "zoom-fit"]',
];
for (const entry of requiredEntries) {
  assert(shortcutTs.includes(entry), `menuShortcuts.ts must include ${entry}`);
}
assert(
  shortcutTs.includes("function matchMenuShortcut"),
  "src/shared/menuShortcuts.ts must define matchMenuShortcut",
);
assert(
  shortcutTs.includes("export function bindWindowsMenuShortcuts"),
  "src/shared/menuShortcuts.ts must export bindWindowsMenuShortcuts",
);

const menuRs = read("src-tauri/src/menu.rs");
const accelerators = [...menuRs.matchAll(/Some\("CmdOrCtrl\+([^"]+)"\)/g)].map(
  (m) => m[1],
);
assert(
  accelerators.length >= 8,
  `expected menu accelerators in menu.rs, got ${String(accelerators.length)}`,
);
assert(
  menuRs.includes("dispatch_menu_action"),
  "menu.rs must expose dispatch_menu_action for the Windows keydown path",
);

const encoder = read("src/encoder/EncoderApp.vue");
const inspector = read("src/inspector/InspectorApp.vue");
assert(
  encoder.includes("bindWindowsMenuShortcuts"),
  "EncoderApp.vue must bind Windows menu shortcuts",
);
assert(
  inspector.includes("bindWindowsMenuShortcuts"),
  "InspectorApp.vue must bind Windows menu shortcuts",
);

console.log(
  `menu-shortcuts-selftest: ok (${String(requiredEntries.length)} lookup entries, ${String(accelerators.length)} accelerators)`,
);
