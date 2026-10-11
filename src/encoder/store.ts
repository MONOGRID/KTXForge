import { useDebounceFn } from "@vueuse/core";
import { computed, reactive, watch } from "vue";
import { call } from "../api/commands";
import type { AnimationOptions } from "../bindings/AnimationOptions";
import type { CreateOptions } from "../bindings/CreateOptions";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { EncodeStage } from "../bindings/EncodeStage";
import type { FieldState } from "../bindings/FieldState";
import type { InputRole } from "../bindings/InputRole";
import type { KtxVersion } from "../bindings/KtxVersion";
import type { OptionKey } from "../bindings/OptionKey";
import type { OptionReport } from "../bindings/OptionReport";
import type { SourceInfo } from "../bindings/SourceInfo";
import {
  applyRoleToOptions,
  isPackRole,
  maxInputsForRole,
  validateArity,
  type PackInputRole,
} from "./inputRole";

export type JobStatus = "idle" | "queued" | "running" | "done" | "error";

export type JobState = {
  status: JobStatus;
  jobId: string | null;
  stage: EncodeStage | null;
  stderrLines: string[];
  result: EncodeResult | null;
  errorMessage: string | null;
};

export type OutputSlot = {
  id: string;
  inputs: SourceInfo[];
  job: JobState;
};

export type BatchQueueState = {
  total: number;
  done: number;
  cancelRequested: boolean;
  runningJobIds: string[];
};

/** Center-panel drop waiting for the user to pick outputs vs pack+role. */
type PendingCenterDrop = {
  sources: SourceInfo[];
};

type EncoderStore = {
  outputs: OutputSlot[];
  selectedId: string | null;
  /** Output id whose Inputs panel was opened via "+" (even with one input). */
  inputsExpandedId: string | null;
  inputRole: InputRole;
  options: CreateOptions;
  report: OptionReport;
  animation: AnimationOptions | null;
  ktxVersion: KtxVersion | null;
  ready: boolean;
  queue: BatchQueueState | null;
  pendingCenterDrop: PendingCenterDrop | null;
};

function emptyJob(): JobState {
  return {
    status: "idle",
    jobId: null,
    stage: null,
    stderrLines: [],
    result: null,
    errorMessage: null,
  };
}

function emptyReport(): OptionReport {
  return {
    fields: [],
    warnings: [],
    sourcesClass: "empty",
    suggestion: null,
  };
}

function newOutputId(): string {
  return crypto.randomUUID();
}

function applySuggestion(report: OptionReport): boolean {
  const suggestion = report.suggestion;
  if (suggestion === null) {
    return false;
  }
  const formatChanged = store.options.format !== suggestion.format;
  const encodeChanged = store.options.encode !== suggestion.encode;
  const convertTfChanged = store.options.color.convertTf !== suggestion.convertTf;
  const assignTfChanged = store.options.color.assignTf !== suggestion.assignTf;
  if (
    !formatChanged &&
    !encodeChanged &&
    !convertTfChanged &&
    !assignTfChanged
  ) {
    return false;
  }
  store.options.format = suggestion.format;
  store.options.encode = suggestion.encode;
  store.options.color.convertTf = suggestion.convertTf;
  store.options.color.assignTf = suggestion.assignTf;
  return true;
}

/** Matches Rust `CreateOptions::default` until `initStore` replaces it. */
function interimOptions(): CreateOptions {
  return {
    format: "R8G8B8A8_SRGB",
    encode: null,
    container: {
      oneD: false,
      cubemap: false,
      layers: null,
      depth: null,
      levels: null,
      generateMipmap: false,
      runtimeMipmap: false,
      width: null,
      height: null,
      scale: 1,
      raw: false,
    },
    filter: {
      mipmapFilter: "lanczos4",
      mipmapFilterScale: 1,
      mipmapWrap: "clamp",
    },
    color: {
      assignTf: null,
      convertTf: null,
      assignPrimaries: null,
      convertPrimaries: null,
      assignTexcoordOrigin: null,
      convertTexcoordOrigin: null,
      failOnColorConversions: false,
      warnOnColorConversions: false,
      noWarnOnColorConversions: false,
      failOnOriginChanges: false,
      warnOnOriginChanges: false,
      swizzle: null,
      inputSwizzle: null,
      normalize: false,
      normalMode: false,
    },
    basisLz: {
      qlevel: 128,
      clevel: 1,
      maxEndpoints: null,
      maxSelectors: null,
      endpointRdoThreshold: 1.25,
      selectorRdoThreshold: 1.25,
      noEndpointRdo: false,
      noSelectorRdo: false,
    },
    uastcLdr: {
      uastcQuality: 1,
      uastcRdo: false,
      uastcRdoL: 1,
      uastcRdoD: 4096,
      uastcRdoB: 10,
      uastcRdoS: 18,
      uastcRdoF: false,
      uastcRdoM: false,
    },
    astc: {
      astcQuality: "medium",
      astcPerceptual: false,
    },
    encoder: {
      threads: 1,
      noSse: false,
      compareSsim: false,
      comparePsnr: false,
    },
    supercompression: { kind: "none" },
  } satisfies CreateOptions;
}

export const store = reactive<EncoderStore>({
  outputs: [],
  selectedId: null,
  inputsExpandedId: null,
  inputRole: "base2D",
  options: interimOptions(),
  report: emptyReport(),
  animation: null,
  ktxVersion: null,
  ready: false,
  queue: null,
  pendingCenterDrop: null,
});

export const selectedOutput = computed((): OutputSlot | null => {
  const id = store.selectedId;
  if (id === null) {
    return null;
  }
  return store.outputs.find((o) => o.id === id) ?? null;
});

/** Bottom Inputs panel: only when packing multiple images into one KTX. */
export const showInputsPanel = computed((): boolean => {
  const selected = selectedOutput.value;
  if (selected === null) {
    return false;
  }
  if (selected.inputs.length > 1) {
    return true;
  }
  return store.inputsExpandedId === selected.id;
});

function ensurePackRole(preferred: PackInputRole = "arrayLayers"): void {
  if (!isPackRole(store.inputRole)) {
    store.inputRole = preferred;
  }
}

function maybeResetToBase2D(): void {
  const anyMulti = store.outputs.some((slot) => slot.inputs.length > 1);
  if (!anyMulti && store.inputsExpandedId === null) {
    store.inputRole = "base2D";
    store.animation = null;
  }
}

/** Sources passed to describe_options: selected inputs, else first output primary. */
function describeSources(): SourceInfo[] {
  const selected = selectedOutput.value;
  if (selected !== null && selected.inputs.length > 0) {
    return selected.inputs;
  }
  if (store.outputs.length === 0) {
    return [];
  }
  const first = store.outputs[0];
  if (first.inputs.length === 0) {
    return [];
  }
  return [first.inputs[0]];
}

function syncRoleOptions(): void {
  const selected = selectedOutput.value;
  const count = selected?.inputs.length ?? 0;
  applyRoleToOptions(store.options, store.inputRole, count);
}

const refreshReport = useDebounceFn(async () => {
  if (!store.ready) {
    return;
  }
  syncRoleOptions();
  try {
    const report = await call("describe_options", {
      options: store.options,
      sources: describeSources(),
      inputRole: store.inputRole,
    });
    store.report = report;
    applySuggestion(report);
  } catch {
    // Keep the last successful report.
  }
}, 80);

watch(
  () =>
    [
      store.outputs.map((o) => o.inputs),
      store.selectedId,
      store.options,
      store.inputRole,
    ] as const,
  () => {
    void refreshReport();
  },
  { deep: true },
);

watch(
  () => store.inputRole,
  () => {
    syncRoleOptions();
    if (store.inputRole !== "arrayLayers" && store.animation !== null) {
      store.animation = null;
    }
  },
);

/** Rust `describe_options` is the only source of enablement. Missing → hidden. */
export function fieldState(key: OptionKey): FieldState {
  const found = store.report.fields.find((field) => field.key === key);
  if (found !== undefined) {
    return found;
  }
  return {
    key,
    enabled: false,
    reason: "waiting for option rules",
    help: "",
    choices: [],
  };
}

export function isOptionEnabled(key: OptionKey): boolean {
  return fieldState(key).enabled;
}

function clearJobOnSlot(slot: OutputSlot): void {
  if (slot.job.status === "running" || slot.job.status === "queued") {
    return;
  }
  slot.job = emptyJob();
}

export function selectOutput(id: string | null): void {
  store.selectedId = id;
}

/** First output's primary input — pins LDR/HDR for the session. */
function sessionAnchor(): SourceInfo | null {
  if (store.outputs.length === 0) {
    return null;
  }
  const first = store.outputs[0];
  if (first.inputs.length === 0) {
    return null;
  }
  return first.inputs[0];
}

async function canAddAsOutput(
  candidate: SourceInfo,
): Promise<string | null> {
  const anchor = sessionAnchor();
  if (anchor === null) {
    return null;
  }
  try {
    await call("check_output_compatible", { anchor, candidate });
    return null;
  } catch (err: unknown) {
    if (err instanceof Error) {
      return err.message;
    }
    return "incompatible with the first KTX output";
  }
}

/** Add one KTX output per source (batch). Returns rejected reasons. */
export async function addOutputsFromSources(
  items: readonly SourceInfo[],
): Promise<string[]> {
  const rejected: string[] = [];
  for (const item of items) {
    const reason = await canAddAsOutput(item);
    if (reason !== null) {
      rejected.push(`${item.name}: ${reason}`);
      continue;
    }
    const slot: OutputSlot = {
      id: newOutputId(),
      inputs: [item],
      job: emptyJob(),
    };
    store.outputs.push(slot);
    store.selectedId = slot.id;
  }
  return rejected;
}

/**
 * Append inputs on the selected output. Entering multi-input switches the
 * session to a pack role (default array layers) and keeps the Inputs panel open.
 */
export function addInputsToSelected(items: readonly SourceInfo[]): string | null {
  const slot = selectedOutput.value;
  if (slot === null) {
    return "Select a KTX output before adding inputs";
  }
  if (items.length === 0) {
    return null;
  }

  clearJobOnSlot(slot);

  const willBeMulti = slot.inputs.length + items.length > 1 || slot.inputs.length > 1;
  if (willBeMulti) {
    ensurePackRole("arrayLayers");
    store.inputsExpandedId = slot.id;
  }

  if (store.inputRole === "base2D") {
    // Single-input replace (still on Base 2D).
    slot.inputs.splice(0, slot.inputs.length, items[0]);
    return items.length > 1
      ? "Only one input kept; choose a pack role to add more"
      : null;
  }

  const max = maxInputsForRole(store.inputRole);
  const room =
    max === null ? items.length : Math.max(0, max - slot.inputs.length);
  if (room === 0) {
    return max === null
      ? "cannot add more inputs"
      : `this role allows at most ${String(max)} inputs`;
  }
  const toAdd = items.slice(0, room);
  for (const item of toAdd) {
    const anchor = sessionAnchor();
    if (anchor !== null && item.kind !== anchor.kind) {
      return `${item.name}: kind must match the first KTX output (${anchor.kind})`;
    }
  }
  slot.inputs.push(...toAdd);
  if (items.length > room) {
    return `only ${String(room)} more input(s) fit this role`;
  }
  return null;
}

/** Create one KTX output packed with all sources under a pack role. */
export async function addPackedOutputFromSources(
  items: readonly SourceInfo[],
  role: PackInputRole,
): Promise<string | null> {
  if (items.length === 0) {
    return null;
  }
  const arityError = validateArity(role, items.length);
  if (arityError !== null) {
    return arityError;
  }
  for (const item of items) {
    const reason = await canAddAsOutput(item);
    if (reason !== null) {
      return `${item.name}: ${reason}`;
    }
  }
  store.inputRole = role;
  const slot: OutputSlot = {
    id: newOutputId(),
    inputs: [...items],
    job: emptyJob(),
  };
  store.outputs.push(slot);
  store.selectedId = slot.id;
  store.inputsExpandedId = slot.id;
  return null;
}

/** Open Inputs for this output (from the "+" control) and pick more files. */
export function beginAddInputs(outputId: string): void {
  const slot = store.outputs.find((o) => o.id === outputId);
  if (slot === undefined) {
    return;
  }
  store.selectedId = outputId;
  store.inputsExpandedId = outputId;
  ensurePackRole("arrayLayers");
}

export function setPendingCenterDrop(sources: readonly SourceInfo[]): void {
  store.pendingCenterDrop = { sources: [...sources] };
}

export function clearPendingCenterDrop(): void {
  store.pendingCenterDrop = null;
}

export function removeOutput(id: string): void {
  const index = store.outputs.findIndex((o) => o.id === id);
  if (index < 0) {
    return;
  }
  const slot = store.outputs[index];
  if (slot.job.status === "running" || slot.job.status === "queued") {
    return;
  }
  store.outputs.splice(index, 1);
  if (store.inputsExpandedId === id) {
    store.inputsExpandedId = null;
  }
  if (store.selectedId === id) {
    store.selectedId = store.outputs[0]?.id ?? null;
  }
  maybeResetToBase2D();
}

export function clearOutputs(): void {
  if (store.queue !== null) {
    return;
  }
  store.outputs.splice(0, store.outputs.length);
  store.selectedId = null;
  store.inputsExpandedId = null;
  store.animation = null;
  store.inputRole = "base2D";
  store.pendingCenterDrop = null;
}

export function removeInput(index: number): void {
  const slot = selectedOutput.value;
  if (slot === null || index < 0 || index >= slot.inputs.length) {
    return;
  }
  clearJobOnSlot(slot);
  slot.inputs.splice(index, 1);
  if (slot.inputs.length <= 1) {
    store.inputsExpandedId = null;
    maybeResetToBase2D();
  }
}

export function moveInput(index: number, delta: -1 | 1): void {
  const slot = selectedOutput.value;
  if (slot === null) {
    return;
  }
  const next = index + delta;
  if (
    index < 0 ||
    next < 0 ||
    index >= slot.inputs.length ||
    next >= slot.inputs.length
  ) {
    return;
  }
  clearJobOnSlot(slot);
  const inputs = slot.inputs;
  const current = inputs[index];
  inputs[index] = inputs[next];
  inputs[next] = current;
}

export function setInputRole(role: InputRole): void {
  store.inputRole = role;
  if (isPackRole(role) && store.selectedId !== null) {
    store.inputsExpandedId = store.selectedId;
  }
  syncRoleOptions();
}

export function allOutputsValid(): boolean {
  if (store.outputs.length === 0) {
    return false;
  }
  return store.outputs.every(
    (slot) => validateArity(store.inputRole, slot.inputs.length) === null,
  );
}

export function outputLabel(slot: OutputSlot): string {
  if (slot.inputs.length === 0) {
    return "Empty output";
  }
  const primary = slot.inputs[0];
  if (slot.inputs.length === 1) {
    return primary.name;
  }
  return `${primary.name} (+${String(slot.inputs.length - 1)})`;
}

export async function initStore(): Promise<void> {
  const [options, ktxVersion] = await Promise.all([
    call("default_create_options", {}),
    call("ktx_version", {}),
  ]);
  store.options = options;
  store.ktxVersion = ktxVersion;
  store.ready = true;
  syncRoleOptions();
  store.report = await call("describe_options", {
    options: store.options,
    sources: describeSources(),
    inputRole: store.inputRole,
  });
}
