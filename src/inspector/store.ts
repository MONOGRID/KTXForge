import { computed, ref, shallowRef } from "vue";
import { call } from "../api/commands";
import { on } from "../api/events";
import { isAppError } from "../api/guards";
import type { AppError } from "../bindings/AppError";
import type { ImageSelection } from "../bindings/ImageSelection";
import type { InspectReport } from "../bindings/InspectReport";
import type { Ktx1Header } from "../bindings/Ktx1Header";
import type { Ktx2Info } from "../bindings/Ktx2Info";
import type { KeyValueValue } from "../bindings/KeyValueValue";

export const MISSING = "—";

const DEFAULT_SELECTION = {
  level: 0,
  face: 0,
  layer: 0,
  depthSlice: 0,
} satisfies ImageSelection;

export const path = ref<string | null>(null);
export const report = shallowRef<InspectReport | null>(null);
export const selection = ref<ImageSelection>({ ...DEFAULT_SELECTION });
export const error = shallowRef<AppError | null>(null);
export const loading = ref(false);

export const ktx2Info = computed((): Ktx2Info | null => {
  const current = report.value;
  if (current === null || current.kind.kind !== "ktx2") {
    return null;
  }
  return current.kind.info;
});

export const ktx1Text = computed((): string | null => {
  const current = report.value;
  if (current === null || current.kind.kind !== "ktx1") {
    return null;
  }
  return current.kind.text;
});

export const ktx1Header = computed((): Ktx1Header | null => {
  const current = report.value;
  if (current === null || current.kind.kind !== "ktx1") {
    return null;
  }
  return current.kind.header;
});

export function displayValue(
  value: string | number | bigint | boolean | null | undefined,
): string {
  if (value === null || value === undefined) {
    return MISSING;
  }
  if (typeof value === "string" && value.length === 0) {
    return MISSING;
  }
  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }
  return String(value);
}

export function displayList(values: readonly string[] | null | undefined): string {
  if (values === null || values === undefined || values.length === 0) {
    return MISSING;
  }
  return values.join(", ");
}

export function displayNumbers(values: readonly number[] | null | undefined): string {
  if (values === null || values === undefined || values.length === 0) {
    return MISSING;
  }
  return values.map(String).join(", ");
}

export function displayKeyValue(value: KeyValueValue): string {
  if (typeof value === "string") {
    return displayValue(value);
  }
  return displayValue(value.hex);
}

async function loadPath(nextPath: string): Promise<void> {
  path.value = nextPath;
  loading.value = true;
  error.value = null;
  try {
    report.value = await call("inspect", { path: nextPath });
    selection.value = { ...DEFAULT_SELECTION };
  } catch (err: unknown) {
    report.value = null;
    error.value = isAppError(err)
      ? err
      : { kind: "io", message: err instanceof Error ? err.message : "Inspect failed" };
  } finally {
    loading.value = false;
  }
}

let listening = false;

/** Wire path from `inspector_current_file` and `inspector://file`. Idempotent. */
export async function initInspectorStore(): Promise<void> {
  if (!listening) {
    listening = true;
    await on("inspector://file", (payload) => {
      void loadPath(payload.path);
    });
  }

  const current = await call("inspector_current_file", {});
  if (current !== null && current.length > 0) {
    await loadPath(current);
  }
}
