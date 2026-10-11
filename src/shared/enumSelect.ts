export type EnumSelectOption = {
  value: string;
  label: string;
  /** Per-value tooltip from Rust `FieldState.choices`. */
  help?: string;
};

export type EnumSelectGroup = {
  label: string;
  options: readonly EnumSelectOption[];
};
