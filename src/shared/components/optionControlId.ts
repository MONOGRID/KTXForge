import type { InjectionKey } from "vue";

/** Provided by `OptionField` for associating labels with Switch/Select/Input. */
export const optionControlIdKey: InjectionKey<string> = Symbol("optionControlId");
