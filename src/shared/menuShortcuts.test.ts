import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { matchMenuShortcut } from "./menuShortcuts";

const root = join(dirname(fileURLToPath(import.meta.url)), "../..");

function read(rel: string): string {
  return readFileSync(join(root, rel), "utf8");
}

function chord(
  code: string,
  opts: { shift?: boolean; meta?: boolean; alt?: boolean; ctrl?: boolean } = {},
): Parameters<typeof matchMenuShortcut>[0] {
  return {
    code,
    ctrlKey: opts.ctrl ?? true,
    metaKey: opts.meta ?? false,
    altKey: opts.alt ?? false,
    shiftKey: opts.shift ?? false,
  };
}

describe("matchMenuShortcut", () => {
  it("maps Ctrl chords to menu ids", () => {
    expect(matchMenuShortcut(chord("KeyO"))).toBe("open");
    expect(matchMenuShortcut(chord("Enter"))).toBe("encode");
    expect(matchMenuShortcut(chord("Period"))).toBe("cancel-encode");
    expect(matchMenuShortcut(chord("KeyS"))).toBe("save");
    expect(matchMenuShortcut(chord("KeyQ"))).toBe("exit");
    expect(matchMenuShortcut(chord("Digit1"))).toBe("zoom-actual");
    expect(matchMenuShortcut(chord("Digit0"))).toBe("zoom-fit");
  });

  it("requires Shift for export and ignores Shift for zoom", () => {
    expect(matchMenuShortcut(chord("KeyE"))).toBeNull();
    expect(matchMenuShortcut(chord("KeyE", { shift: true }))).toBe("export");
    expect(matchMenuShortcut(chord("Equal", { shift: true }))).toBe("zoom-in");
    expect(matchMenuShortcut(chord("Minus", { shift: true }))).toBe("zoom-out");
    expect(matchMenuShortcut(chord("NumpadAdd"))).toBe("zoom-in");
    expect(matchMenuShortcut(chord("NumpadSubtract"))).toBe("zoom-out");
  });

  it("ignores non-Ctrl-only modifiers", () => {
    expect(matchMenuShortcut(chord("KeyO", { ctrl: false }))).toBeNull();
    expect(matchMenuShortcut(chord("KeyO", { meta: true }))).toBeNull();
    expect(matchMenuShortcut(chord("KeyO", { alt: true }))).toBeNull();
  });
});

describe("menu shortcut wiring", () => {
  it("keeps SHORTCUT_LOOKUP entries, menu.rs accelerators, and app binds in sync", () => {
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
      expect(shortcutTs, `missing ${entry}`).toContain(entry);
    }
    expect(shortcutTs).toContain("export function matchMenuShortcut");
    expect(shortcutTs).toContain("export function bindWindowsMenuShortcuts");

    const menuRs = read("src-tauri/src/menu.rs");
    const accelerators = [
      ...menuRs.matchAll(/Some\("CmdOrCtrl\+([^"]+)"\)/g),
    ].map((m) => m[1]);
    expect(accelerators.length).toBeGreaterThanOrEqual(8);
    expect(menuRs).toContain("dispatch_menu_action");

    expect(read("src/encoder/EncoderApp.vue")).toContain(
      "bindWindowsMenuShortcuts",
    );
    expect(read("src/inspector/InspectorApp.vue")).toContain(
      "bindWindowsMenuShortcuts",
    );
  });
});
