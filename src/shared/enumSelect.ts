import type { Component } from "vue";

export type EnumSelectOption = {
  value: string;
  label: string;
  /** Per-value tooltip from Rust `FieldState.choices`. */
  help?: string;
  /** Optional leading Lucide (or other) icon in the menu and trigger. */
  icon?: Component;
};

export type EnumSelectGroup = {
  label: string;
  options: readonly EnumSelectOption[];
};
