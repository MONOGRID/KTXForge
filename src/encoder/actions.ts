import { call } from "../api/commands";
import { toAppError } from "../api/guards";
import { toastAppError } from "../shared/toastAppError";
import {
  addSources,
  resetSources,
  store,
  type JobState,
  type JobStatus,
} from "./store";

function isJobStatus(status: JobStatus, expected: JobStatus): boolean {
  return status === expected;
}

export function canEncode(): boolean {
  return store.sources.length > 0 && !isJobStatus(store.job.status, "running");
}

export function canCancelEncode(): boolean {
  return isJobStatus(store.job.status, "running") && store.job.jobId !== null;
}

export function canSaveEncoded(): boolean {
  return (
    isJobStatus(store.job.status, "done") &&
    store.job.jobId !== null &&
    store.job.result !== null
  );
}

export function canClearSources(): boolean {
  return store.sources.length > 0 && !isJobStatus(store.job.status, "running");
}

function resetJobForRun(jobId: string): void {
  const next: JobState = {
    status: "running",
    jobId,
    stage: "converting",
    stderrLines: [],
    result: null,
  };
  store.job = next;
}

export async function openSources(): Promise<void> {
  try {
    const picked = await call("pick_sources", {});
    if (picked.length > 0) {
      addSources(picked);
    }
  } catch (err: unknown) {
    toastAppError(toAppError(err));
  }
}

export async function encode(): Promise<void> {
  if (!canEncode()) {
    return;
  }

  const jobId = crypto.randomUUID();
  resetJobForRun(jobId);

  try {
    const result = await call("encode", {
      request: {
        jobId,
        sources: store.sources.map((source) => source.path),
        options: store.options,
        animation: store.animation,
      },
    });
    if (store.job.jobId !== jobId) {
      return;
    }
    store.job.status = "done";
    store.job.stage = "done";
    store.job.result = result;
  } catch (err: unknown) {
    if (store.job.jobId !== jobId) {
      return;
    }
    const appError = toAppError(err);
    store.job.status = appError.kind === "cancelled" ? "idle" : "error";
    store.job.result = null;
    if (appError.kind === "cancelled") {
      store.job.stage = null;
    } else {
      toastAppError(appError);
    }
  }
}

export async function cancelEncode(): Promise<void> {
  const jobId = store.job.jobId;
  if (jobId === null || !canCancelEncode()) {
    return;
  }
  try {
    await call("cancel_encode", { jobId });
  } catch (err: unknown) {
    toastAppError(toAppError(err));
  }
}

export async function saveEncoded(): Promise<void> {
  const jobId = store.job.jobId;
  if (jobId === null || !canSaveEncoded()) {
    return;
  }
  try {
    await call("save_encoded", { jobId });
  } catch (err: unknown) {
    toastAppError(toAppError(err));
  }
}

export function clearSources(): void {
  if (!canClearSources()) {
    return;
  }
  resetSources();
}
