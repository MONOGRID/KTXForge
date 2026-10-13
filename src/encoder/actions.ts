import { call } from "../api/commands";
import { toAppError } from "../api/guards";
import type { EncodeResult } from "../bindings/EncodeResult";
import type { SourceInfo } from "../bindings/SourceInfo";
import {
  alertAppError,
  alertErrorText,
  alertInvalidInput,
  notifyText,
} from "../shared/alertAppError";
import { formatAppError } from "../shared/formatAppError";
import type { DropZone } from "./dropTarget";
import type { PackInputRole } from "./inputRole";
import {
  addInputsToSelected,
  addOutputsFromSources,
  addPackedOutputFromSources,
  allOutputsValid,
  beginAddInputs,
  clearOutputs,
  clearPendingCenterDrop,
  outputLabel,
  selectedOutput,
  setPendingCenterDrop,
  store,
  type BatchQueueState,
  type OutputSlot,
} from "./store";

/** Concurrent `ktx create` processes during Encode all. */
const ENCODE_PARALLELISM = 2;

function animationBlocksEncode(): boolean {
  return store.outputs.some(
    (slot) =>
      slot.animation !== null &&
      (slot.role !== "arrayLayers" || slot.inputs.length < 2),
  );
}

export function canEncode(): boolean {
  return (
    store.queue === null &&
    allOutputsValid() &&
    store.report.suggestion === null &&
    !animationBlocksEncode()
  );
}

export function canCancelEncode(): boolean {
  return store.queue !== null;
}

export function canSaveEncoded(): boolean {
  if (store.queue !== null) {
    return false;
  }
  const selected = selectedOutput.value;
  return (
    selected !== null &&
    selected.job.status === "done" &&
    selected.job.jobId !== null &&
    selected.job.result !== null
  );
}

export function canSaveAllEncoded(): boolean {
  if (store.queue !== null) {
    return false;
  }
  return store.outputs.some(
    (slot) =>
      slot.job.status === "done" &&
      slot.job.jobId !== null &&
      slot.job.result !== null,
  );
}

export function canClearSources(): boolean {
  return store.outputs.length > 0 && store.queue === null;
}

export async function openSourcesAsOutputs(): Promise<void> {
  try {
    const picked = await call("pick_sources", {});
    if (picked.length === 0) {
      return;
    }
    const rejected = await addOutputsFromSources(picked);
    if (rejected.length > 0) {
      await alertInvalidInput(rejected.join("; "));
    }
  } catch (err: unknown) {
    await alertAppError(toAppError(err));
  }
}

export async function openSourcesAsInputs(): Promise<void> {
  if (selectedOutput.value === null) {
    await alertInvalidInput("Select a KTX output before adding inputs");
    return;
  }
  try {
    const picked = await call("pick_sources", {});
    if (picked.length === 0) {
      return;
    }
    const notice = await addInputsToSelected(picked);
    if (notice !== null) {
      await alertInvalidInput(notice);
    }
  } catch (err: unknown) {
    await alertAppError(toAppError(err));
  }
}

/** "+" on a KTX output row: open Inputs and pick more images for that output. */
export async function addInputsForOutput(outputId: string): Promise<void> {
  beginAddInputs(outputId);
  await openSourcesAsInputs();
}

function applyEncodeSuccess(
  slot: OutputSlot,
  jobId: string,
  result: EncodeResult,
): void {
  if (slot.job.jobId !== jobId) {
    return;
  }
  slot.job.status = "done";
  slot.job.stage = "done";
  slot.job.result = result;
}

function applyEncodeFailure(slot: OutputSlot, jobId: string, err: unknown): void {
  if (slot.job.jobId !== jobId) {
    return;
  }
  const appError = toAppError(err);
  if (appError.kind === "cancelled") {
    slot.job.status = "idle";
    slot.job.stage = null;
    slot.job.jobId = null;
    slot.job.errorMessage = null;
    return;
  }
  slot.job.status = "error";
  slot.job.result = null;
  const message = formatAppError(appError);
  slot.job.errorMessage = message;
  console.error(`[ktxforge] encode failed (${outputLabel(slot)}):\n${message}`);
}

async function encodeOne(slot: OutputSlot): Promise<void> {
  const jobId = crypto.randomUUID();
  slot.job = {
    status: "running",
    jobId,
    stage: "converting",
    result: null,
    errorMessage: null,
  };

  try {
    const result = await call("encode", {
      request: {
        jobId,
        outputId: slot.id,
        sources: slot.inputs.map((source) => source.path),
        options: store.options,
        animation: slot.role === "arrayLayers" ? slot.animation : null,
        inputRole: slot.role,
      },
    });
    applyEncodeSuccess(slot, jobId, result);
  } catch (err: unknown) {
    applyEncodeFailure(slot, jobId, err);
  }
}

function markSlotsQueued(slots: readonly OutputSlot[]): void {
  for (const slot of slots) {
    slot.job = {
      status: "queued",
      jobId: null,
      stage: null,
      result: null,
      errorMessage: null,
    };
  }
}

function resetStillQueued(slots: readonly OutputSlot[]): void {
  for (const slot of slots) {
    if (slot.job.status === "queued") {
      slot.job = {
        status: "idle",
        jobId: null,
        stage: null,
        result: null,
        errorMessage: null,
      };
    }
  }
}

async function runEncodeWorkers(
  slots: readonly OutputSlot[],
  queueState: BatchQueueState,
): Promise<void> {
  let nextIndex = 0;
  const runWorker = async (): Promise<void> => {
    while (!queueState.cancelRequested) {
      const index = nextIndex;
      nextIndex += 1;
      if (index >= slots.length) {
        return;
      }
      await encodeOne(slots[index]);
      queueState.done += 1;
    }
  };
  await Promise.all(
    Array.from(
      { length: Math.min(ENCODE_PARALLELISM, slots.length) },
      () => runWorker(),
    ),
  );
}

export async function encode(): Promise<void> {
  if (!canEncode()) {
    return;
  }

  const slots = store.outputs.filter((slot) => slot.inputs.length > 0);
  if (slots.length === 0) {
    return;
  }

  const queueState: BatchQueueState = {
    total: slots.length,
    done: 0,
    cancelRequested: false,
  };
  store.queue = queueState;
  markSlotsQueued(slots);
  await runEncodeWorkers(slots, queueState);
  store.queue = null;

  if (queueState.cancelRequested) {
    resetStillQueued(slots);
    return;
  }

  await notifyEncodeFinished(slots);
}

function formatEncodeFailureBody(
  errors: readonly OutputSlot[],
  total: number,
): string {
  if (errors.length === 1) {
    return errors[0].job.errorMessage ?? "Encode failed";
  }
  return [
    `${String(errors.length)} of ${String(total)} encodes failed`,
    ...errors.map((slot) => {
      const detail = slot.job.errorMessage ?? "Encode failed";
      return `${outputLabel(slot)}:\n${detail}`;
    }),
  ].join("\n\n");
}

async function notifyEncodeFinished(
  slots: readonly OutputSlot[],
): Promise<void> {
  const errors = slots.filter((s) => s.job.status === "error");
  if (errors.length > 0) {
    await alertErrorText(
      formatEncodeFailureBody(errors, slots.length),
      "Encode failed",
    );
    return;
  }

  const done = slots.filter((s) => s.job.status === "done").length;
  if (done === 0) {
    return;
  }
  const body =
    done === 1 ? "1 texture encoded" : `${String(done)} textures encoded`;
  await notifyText("Encode complete", body);
}

export async function cancelEncode(): Promise<void> {
  const queue = store.queue;
  if (queue === null) {
    return;
  }
  queue.cancelRequested = true;
  const runningIds = store.outputs
    .filter((s) => s.job.status === "running" && s.job.jobId !== null)
    .map((s) => s.job.jobId)
    .filter((id): id is string => id !== null);
  for (const jobId of runningIds) {
    try {
      await call("cancel_encode", { jobId });
    } catch (err: unknown) {
      await alertAppError(toAppError(err));
    }
  }
}

export async function saveEncoded(): Promise<void> {
  const selected = selectedOutput.value;
  if (selected === null || !canSaveEncoded()) {
    return;
  }
  const jobId = selected.job.jobId;
  if (jobId === null) {
    return;
  }
  try {
    await call("save_encoded", { jobId });
  } catch (err: unknown) {
    await alertAppError(toAppError(err));
  }
}

export async function saveAllEncoded(): Promise<void> {
  if (!canSaveAllEncoded()) {
    return;
  }
  const jobIds = store.outputs
    .filter(
      (slot) =>
        slot.job.status === "done" &&
        slot.job.jobId !== null &&
        slot.job.result !== null,
    )
    .map((slot) => slot.job.jobId)
    .filter((id): id is string => id !== null);
  if (jobIds.length === 0) {
    return;
  }
  if (jobIds.length === 1) {
    try {
      await call("save_encoded", { jobId: jobIds[0] });
    } catch (err: unknown) {
      await alertAppError(toAppError(err));
    }
    return;
  }
  try {
    await call("save_encoded_batch", { jobIds });
  } catch (err: unknown) {
    await alertAppError(toAppError(err));
  }
}

export function clearSources(): void {
  if (!canClearSources()) {
    return;
  }
  clearOutputs();
}

async function dropOntoInputs(sources: readonly SourceInfo[]): Promise<void> {
  if (selectedOutput.value === null) {
    await alertInvalidInput(
      "Select a KTX output (or use +) before dropping inputs",
    );
    return;
  }
  const notice = await addInputsToSelected(sources);
  if (notice !== null) {
    await alertInvalidInput(notice);
  }
}

async function dropOntoOutputs(sources: readonly SourceInfo[]): Promise<void> {
  const rejected = await addOutputsFromSources(sources);
  if (rejected.length > 0) {
    await alertInvalidInput(rejected.join("; "));
  }
}

/** Route dropped images by active drop zone. */
export async function handleDroppedSources(
  sources: readonly SourceInfo[],
  zone: DropZone | null,
): Promise<void> {
  if (sources.length === 0) {
    return;
  }
  // null / center: ask via the preview dialog (chrome drops count as center).
  // One image has no pack choice — go straight to a separate KTX output.
  if (zone === null || zone === "center") {
    if (sources.length === 1) {
      await dropOntoOutputs(sources);
      return;
    }
    setPendingCenterDrop(sources);
    return;
  }
  if (zone === "inputs") {
    await dropOntoInputs(sources);
    return;
  }
  await dropOntoOutputs(sources);
}

export async function applyCenterDropAsOutputs(): Promise<void> {
  const pending = store.pendingCenterDrop;
  if (pending === null) {
    return;
  }
  const sources = pending.sources;
  clearPendingCenterDrop();
  const rejected = await addOutputsFromSources(sources);
  if (rejected.length > 0) {
    await alertInvalidInput(rejected.join("; "));
  }
}

export async function applyCenterDropAsPacked(
  role: PackInputRole,
): Promise<void> {
  const pending = store.pendingCenterDrop;
  if (pending === null) {
    return;
  }
  const sources = pending.sources;
  clearPendingCenterDrop();
  const error = await addPackedOutputFromSources(sources, role);
  if (error !== null) {
    await alertInvalidInput(error);
  }
}

export function slotByJobId(jobId: string): OutputSlot | undefined {
  return store.outputs.find((s) => s.job.jobId === jobId);
}
