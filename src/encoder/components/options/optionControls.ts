import { computed, type ComputedRef } from "vue";
import type { OptionKey } from "../../../bindings/OptionKey";
import type { OptionWarning } from "../../../bindings/OptionWarning";
import { store } from "../../store";

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
  get: () => number | null;
  set: (value: number | null) => void;
};

type EnumOptionField = {
  key: OptionKey;
  label: string;
  kind: "enum";
  labels: Record<string, string>;
  get: () => string | null;
  set: (value: string | null) => void;
};

type TextOptionField = {
  key: OptionKey;
  label: string;
  kind: "text";
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

export function enumEntries(
  labels: Record<string, string>,
): { value: string; label: string }[] {
  return Object.keys(labels).map((value) => ({
    value,
    label: labels[value] ?? value,
  }));
}

function inputValue(event: Event): string {
  const target = event.target;
  if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement) {
    return target.value;
  }
  return "";
}

export function checkboxChecked(event: Event): boolean {
  const target = event.target;
  return target instanceof HTMLInputElement && target.checked;
}

function isValidOptionalNumber(value: number, integer: boolean): boolean {
  return Number.isFinite(value) && (!integer || Number.isInteger(value));
}

function parseOptionalNumber(raw: string, integer: boolean): number | null {
  const trimmed = raw.trim();
  if (trimmed === "") {
    return null;
  }
  const value = Number(trimmed);
  return isValidOptionalNumber(value, integer) ? value : null;
}

export function onEnumChange(field: EnumOptionField, event: Event): void {
  const raw = inputValue(event);
  field.set(raw === "" ? null : raw);
}

export function onNumberChange(field: NumberOptionField, event: Event): void {
  const raw = inputValue(event);
  if (raw.trim() === "") {
    field.set(null);
    return;
  }
  const parsed = parseOptionalNumber(raw, field.step === "1");
  if (parsed !== null) {
    field.set(parsed);
  }
}

export function onTextChange(field: TextOptionField, event: Event): void {
  const raw = inputValue(event).trim();
  field.set(raw === "" ? null : raw);
}

export function isLabeled<E extends string>(
  labels: Record<E, string>,
  value: string,
): value is E {
  return Object.prototype.hasOwnProperty.call(labels, value);
}
