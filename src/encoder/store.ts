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
import type { RoleArity } from "../bindings/RoleArity";
import type { SourceInfo } from "../bindings/SourceInfo";
import { isPackRole, roleLabel, type PackInputRole } from "./inputRole";

export type JobStatus = "idle" | "queued" | "running" | "done" | "error";

export type JobState = {
  status: JobStatus;
  jobId: string | null;
  stage: EncodeStage | null;
  result: EncodeResult | null;
  errorMessage: string | null;
};

export type OutputSlot = {
  id: string;
  inputs: SourceInfo[];
  /** How `inputs` map onto the KTX (single image, layers, cubemap, …). */
  role: InputRole;
  /** KTXanimData for an array-layers output; `null` = no animation. */
  animation: AnimationOptions | null;
  job: JobState;
};

export type BatchQueueState = {
  total: number;
  done: number;
  cancelRequested: boolean;
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
  options: CreateOptions;
  report: OptionReport;
  /** Input-count bounds per role, from Rust `input_role_arities`. */
  arities: RoleArity[];
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
    result: null,
    errorMessage: null,
  };
}

function emptyReport(): OptionReport {
  return {
    fields: [],
    warnings: [],
    sourcesClass: "empty",
    defaultFormat: "R8G8B8A8_SRGB",
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
  options: interimOptions(),
  report: emptyReport(),
  arities: [],
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

/** Role of the selected output (Base 2D when nothing is selected). */
export const selectedRole = computed(
  (): InputRole => selectedOutput.value?.role ?? "base2D",
);

function setSlotRole(slot: OutputSlot, role: InputRole): void {
  slot.role = role;
  if (role !== "arrayLayers") {
    slot.animation = null;
  }
}

function ensurePackRole(slot: OutputSlot): void {
  if (!isPackRole(slot.role)) {
    setSlotRole(slot, "arrayLayers");
  }
}

/** Back to a single image: drop the pack role unless Inputs was opened with "+". */
function maybeResetToBase2D(slot: OutputSlot): void {
  if (slot.inputs.length <= 1 && store.inputsExpandedId !== slot.id) {
    setSlotRole(slot, "base2D");
  }
}

function arityFor(role: InputRole): RoleArity | undefined {
  return store.arities.find((arity) => arity.role === role);
}

function fitsArity(role: InputRole, count: number): boolean {
  const arity = arityFor(role);
  if (arity === undefined) {
    return false;
  }
  return count >= arity.min && (arity.max === null || count <= arity.max);
}

function arityNotice(role: InputRole, count: number): string | null {
  if (fitsArity(role, count)) {
    return null;
  }
  const arity = arityFor(role);
  if (arity === undefined) {
    return "waiting for input rules";
  }
  const expected =
    arity.max === arity.min
      ? `exactly ${String(arity.min)}`
      : `at least ${String(arity.min)}`;
  return `${roleLabel(role)} expects ${expected} input image(s), got ${String(count)}`;
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

function describeOptions(): Promise<OptionReport> {
  return call("describe_options", {
    options: store.options,
    sources: describeSources(),
    inputRole: selectedRole.value,
  });
}

/** Latest request wins; an older response arriving late is dropped. */
let reportSeq = 0;

const refreshReport = useDebounceFn(async () => {
  if (!store.ready) {
    return;
  }
  const seq = ++reportSeq;
  try {
    const report = await describeOptions();
    if (seq !== reportSeq) {
      return;
    }
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
      selectedRole.value,
    ] as const,
  () => {
    void refreshReport();
  },
  { deep: true },
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
      role: "base2D",
      animation: null,
      job: emptyJob(),
    };
    store.outputs.push(slot);
    store.selectedId = slot.id;
  }
  return rejected;
}

/** First item Rust refuses to mix with the session anchor (LDR vs HDR), as a notice. */
async function firstIncompatible(
  items: readonly SourceInfo[],
): Promise<string | null> {
  for (const item of items) {
    const reason = await canAddAsOutput(item);
    if (reason !== null) {
      return `${item.name}: ${reason}`;
    }
  }
  return null;
}

function roomForRole(slot: OutputSlot, want: number): {
  room: number;
  fullMessage: string | null;
} {
  const max = arityFor(slot.role)?.max ?? null;
  if (max === null) {
    return { room: want, fullMessage: null };
  }
  const room = Math.max(0, max - slot.inputs.length);
  return {
    room,
    fullMessage:
      room === 0 ? `this role allows at most ${String(max)} inputs` : null,
  };
}

async function appendPackInputs(
  slot: OutputSlot,
  items: readonly SourceInfo[],
): Promise<string | null> {
  const { room, fullMessage } = roomForRole(slot, items.length);
  if (room === 0) {
    return fullMessage ?? "cannot add more inputs";
  }
  const toAdd = items.slice(0, room);
  const kindError = await firstIncompatible(toAdd);
  if (kindError !== null) {
    return kindError;
  }
  slot.inputs.push(...toAdd);
  if (items.length > room) {
    return `only ${String(room)} more input(s) fit this role`;
  }
  return null;
}

function promoteToPackIfMulti(slot: OutputSlot, items: readonly SourceInfo[]): void {
  const willBeMulti = slot.inputs.length + items.length > 1 || slot.inputs.length > 1;
  if (!willBeMulti) {
    return;
  }
  ensurePackRole(slot);
  store.inputsExpandedId = slot.id;
}

function replaceBase2DInput(
  slot: OutputSlot,
  items: readonly SourceInfo[],
): string | null {
  slot.inputs.splice(0, slot.inputs.length, items[0]);
  return items.length > 1
    ? "Only one input kept; choose a pack role to add more"
    : null;
}

/**
 * Append inputs on the selected output. Entering multi-input switches the
 * session to a pack role (default array layers) and keeps the Inputs panel open.
 */
export async function addInputsToSelected(
  items: readonly SourceInfo[],
): Promise<string | null> {
  const slot = selectedOutput.value;
  if (slot === null) {
    return "Select a KTX output before adding inputs";
  }
  if (items.length === 0) {
    return null;
  }

  clearJobOnSlot(slot);
  promoteToPackIfMulti(slot, items);

  if (slot.role === "base2D") {
    return replaceBase2DInput(slot, items);
  }

  return appendPackInputs(slot, items);
}

/** Create one KTX output packed with all sources under a pack role. */
export async function addPackedOutputFromSources(
  items: readonly SourceInfo[],
  role: PackInputRole,
): Promise<string | null> {
  if (items.length === 0) {
    return null;
  }
  const arityError = arityNotice(role, items.length);
  if (arityError !== null) {
    return arityError;
  }
  const kindError = await firstIncompatible(items);
  if (kindError !== null) {
    return kindError;
  }
  const slot: OutputSlot = {
    id: newOutputId(),
    inputs: [...items],
    role,
    animation: null,
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
  ensurePackRole(slot);
}

export function setPendingCenterDrop(sources: readonly SourceInfo[]): void {
  store.pendingCenterDrop = { sources: [...sources] };
}

export function clearPendingCenterDrop(): void {
  store.pendingCenterDrop = null;
}

function canRemoveSlot(slot: OutputSlot): boolean {
  return slot.job.status !== "running" && slot.job.status !== "queued";
}

function forgetOutputIds(id: string): void {
  if (store.inputsExpandedId === id) {
    store.inputsExpandedId = null;
  }
  if (store.selectedId === id) {
    store.selectedId = store.outputs[0]?.id ?? null;
  }
}

/** Rust deletes the encoded files kept for outputs that left the queue. */
function discardEncodes(outputIds: string[]): void {
  if (outputIds.length === 0) {
    return;
  }
  void call("discard_outputs", { outputIds }).catch(() => {
    // Temp files are still removed when the app exits.
  });
}

export function removeOutput(id: string): void {
  const index = store.outputs.findIndex((o) => o.id === id);
  if (index < 0) {
    return;
  }
  if (!canRemoveSlot(store.outputs[index])) {
    return;
  }
  store.outputs.splice(index, 1);
  forgetOutputIds(id);
  discardEncodes([id]);
}

export function clearOutputs(): void {
  if (store.queue !== null) {
    return;
  }
  discardEncodes(store.outputs.map((slot) => slot.id));
  store.outputs.splice(0, store.outputs.length);
  store.selectedId = null;
  store.inputsExpandedId = null;
  store.pendingCenterDrop = null;
}

export function removeInput(index: number): void {
  const slot = selectedOutput.value;
  if (slot === null || index < 0 || index >= slot.inputs.length) {
    return;
  }
  clearJobOnSlot(slot);
  slot.inputs.splice(index, 1);
  if (slot.inputs.length < 2) {
    slot.animation = null;
  }
  if (slot.inputs.length <= 1) {
    if (store.inputsExpandedId === slot.id) {
      store.inputsExpandedId = null;
    }
    maybeResetToBase2D(slot);
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

export function setInputRole(role: PackInputRole): void {
  const slot = selectedOutput.value;
  if (slot === null) {
    return;
  }
  clearJobOnSlot(slot);
  setSlotRole(slot, role);
  store.inputsExpandedId = slot.id;
}

export function allOutputsValid(): boolean {
  if (store.outputs.length === 0) {
    return false;
  }
  return store.outputs.every((slot) => fitsArity(slot.role, slot.inputs.length));
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
  const [options, ktxVersion, arities] = await Promise.all([
    call("default_create_options", {}),
    call("ktx_version", {}),
    call("input_role_arities", {}),
  ]);
  store.options = options;
  store.ktxVersion = ktxVersion;
  store.arities = arities;
  store.ready = true;
  const seq = ++reportSeq;
  const report = await describeOptions();
  if (seq === reportSeq) {
    store.report = report;
  }
}
