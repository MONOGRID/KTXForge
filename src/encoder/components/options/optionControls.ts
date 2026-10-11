import { computed, type ComputedRef } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import type { OptionWarning } from "../../../bindings/OptionWarning";
import { isOptionEnabled, store } from "../../store";

type BoolOptionField = {
  key: OptionKey;
  label: string;
  kind: "bool";
  get: () => boolean;
  set: (value: boolean) => void;
};

type NumberOptionField = {
  key: OptionKey;
  label: string;
  kind: "number";
  step: string;
  placeholder?: string;
  get: () => number | null;
  set: (value: number | null) => void;
};

type EnumOptionField = {
  key: OptionKey;
  label: string;
  kind: "enum";
  labels: Record<string, string>;
  /** Omit / `false` = no empty choice (field has a real default). */
  emptyLabel?: string | false;
  get: () => string | null;
  set: (value: string | null) => void;
};

type TextOptionField = {
  key: OptionKey;
  label: string;
  kind: "text";
  /** Format hint only (e.g. `rgba`); never a stand-in for a default value. */
  placeholder?: string;
  get: () => string | null;
  set: (value: string | null) => void;
};

export type OptionControlField =
  | BoolOptionField
  | NumberOptionField
  | EnumOptionField
  | TextOptionField;

function appendWarning(
  map: Map<OptionKey, OptionWarning[]>,
  sectionKeys: Set<OptionKey>,
  warning: OptionWarning,
): void {
  const key = warning.key;
  if (key === null || !sectionKeys.has(key)) {
    return;
  }
  const list = map.get(key) ?? [];
  list.push(warning);
  map.set(key, list);
}

function collectWarnings(
  sectionKeys: Set<OptionKey>,
): Map<OptionKey, OptionWarning[]> {
  const map = new Map<OptionKey, OptionWarning[]>();
  for (const warning of store.report.warnings) {
    appendWarning(map, sectionKeys, warning);
  }
  return map;
}

export function warningsForFields(
  fields: () => readonly OptionControlField[],
): ComputedRef<Map<OptionKey, OptionWarning[]>> {
  return computed(() =>
    collectWarnings(new Set(fields().map((field) => field.key))),
  );
}

export function anyEnabled(keys: readonly OptionKey[]): boolean {
  return keys.some((key) => isOptionEnabled(key));
}

export function enumEntries(
  labels: Record<string, string>,
): { value: string; label: string }[] {
  return Object.keys(labels).map((value) => ({
    value,
    label: labels[value] ?? value,
  }));
}

export function isLabeled<E extends string>(
  labels: Record<E, string>,
  value: string,
): value is E {
  return Object.prototype.hasOwnProperty.call(labels, value);
}

export function parseNumberField(raw: string, integer: boolean): number | null {
  const trimmed = raw.trim();
  if (trimmed === "") {
    return null;
  }
  const value = Number(trimmed);
  if (!Number.isFinite(value)) {
    return null;
  }
  return integer ? Math.trunc(value) : value;
}
