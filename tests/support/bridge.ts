/**
 * Runs the real frontend (Vite dev server) in Chromium against the real Rust
 * commands: `window.__TAURI_INTERNALS__` forwards every `invoke` to
 * `ktxforge-e2e`. Only window chrome that has no webview equivalent (native
 * menus, dialogs, notifications, window focus) is answered here.
 */
import type { Page } from "@playwright/test";
import type { MenuAction } from "../../src/bindings/MenuAction";
import type { Ipc, Json, Outcome } from "./ipc";

type Answer = { ok: unknown } | { err: unknown };

declare global {
  interface Window {
    /** Exposed by Playwright: forwards one IPC call to Node. */
    __e2eInvoke?: (cmd: string, args: unknown) => Promise<Answer>;
    /** Delivers an app event to the page's listeners, as Rust's `emit` would. */
    __e2eEmit?: (event: string, payload: unknown) => void;
    __TAURI_INTERNALS__?: unknown;
  }
}

/** Commands answered without Rust, by name (a stub may return a Promise). */
export type Stubs = Record<string, (args: unknown) => unknown>;

const DEFAULT_STUBS: Stubs = {
  set_menu_item_enabled: () => null,
  "plugin:window|is_focused": () => true,
  "plugin:window|get_all_windows": () => ["main"],
  "plugin:notification|is_permission_granted": () => true,
};

/** Installed before any app script runs; plain browser code. */
function installTauriInternals(label: string): void {
  type Handler = (payload: unknown) => void;
  const callbacks = new Map<number, Handler>();
  const listeners = new Map<string, number[]>();
  let nextId = 1;

  function toBytes(value: unknown): unknown {
    if (typeof value === "object" && value !== null && "bytes" in value && typeof value.bytes === "string") {
      const binary = atob(value.bytes);
      const out = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
      return out.buffer;
    }
    return value;
  }

  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label }, currentWebview: { windowLabel: label, label } },
    transformCallback(callback: Handler | undefined, once = false): number {
      const id = nextId++;
      callbacks.set(id, (payload) => {
        if (once) callbacks.delete(id);
        callback?.(payload);
      });
      return id;
    },
    unregisterCallback(id: number): void {
      callbacks.delete(id);
    },
    async invoke(cmd: string, args: Record<string, unknown> = {}): Promise<unknown> {
      if (cmd === "plugin:event|listen") {
        const event = String(args.event);
        const handler = Number(args.handler);
        listeners.set(event, [...(listeners.get(event) ?? []), handler]);
        return handler;
      }
      if (cmd === "plugin:event|unlisten") {
        const event = String(args.event);
        listeners.set(event, (listeners.get(event) ?? []).filter((id) => id !== Number(args.eventId)));
        return null;
      }
      const forward = window.__e2eInvoke;
      if (forward === undefined) throw new Error("e2e bridge is not exposed");
      const answer = await forward(cmd, args);
      if ("err" in answer) throw answer.err;
      return toBytes(answer.ok);
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener(event: string, id: number): void {
      callbacks.delete(id);
      listeners.set(event, (listeners.get(event) ?? []).filter((l) => l !== id));
    },
  };
  window.__e2eEmit = (event: string, payload: unknown): void => {
    for (const id of listeners.get(event) ?? []) {
      callbacks.get(id)?.({ event, id, payload });
    }
  };
}

function answer(outcome: Outcome<unknown>): Answer {
  return outcome.ok ? { ok: outcome.value } : { err: outcome.error };
}

/** Route this page's IPC to `ipc`. Call before `page.goto`. */
export async function bridge(page: Page, ipc: Ipc, label: string, stubs: Stubs = {}): Promise<void> {
  const local = { ...DEFAULT_STUBS, ...stubs };
  await page.exposeFunction("__e2eInvoke", async (cmd: string, args: unknown): Promise<Answer> => {
    const stub = Object.entries(local).find(([name]) => name === cmd)?.[1];
    if (stub !== undefined) return { ok: await stub(args) };
    if (cmd.startsWith("plugin:")) return { ok: null };
    return answer(await ipc.invokeAny(cmd, args));
  });
  await page.addInitScript(installTauriInternals, label);
}

/** Deliver a native-menu click (`menu://action`), as Rust emits it. */
export async function menu(page: Page, action: MenuAction): Promise<void> {
  await emit(page, "menu://action", action);
}

async function emit(page: Page, event: string, payload: Json): Promise<void> {
  // Sent as text: Playwright's serializable-argument types recurse too deep on JSON.
  await page.evaluate(
    ({ name, data }) => {
      window.__e2eEmit?.(name, JSON.parse(data));
    },
    { name: event, data: JSON.stringify(payload) },
  );
}
