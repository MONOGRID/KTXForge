<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { call } from "../../api/commands";
import { on } from "../../api/events";
import { toAppError } from "../../api/guards";
import type { EncodeStage } from "../../bindings/EncodeStage";
import Button from "../../shared/components/Button.vue";
import ErrorBox from "../../shared/components/ErrorBox.vue";
import ProgressBar from "../../shared/components/ProgressBar.vue";
import type { JobState, JobStatus } from "../store";
import { store } from "../store";

const STAGE_LABELS = {
  converting: "Converting…",
  encoding: "Encoding…",
  "writing-anim": "Writing animation…",
  previewing: "Building preview…",
  done: "Done",
} as const satisfies Record<EncodeStage, string>;

const STDERR_KEEP = 40;

function isJobStatus(status: JobStatus, expected: JobStatus): boolean {
  return status === expected;
}

const isRunning = computed(
  (): boolean => isJobStatus(store.job.status, "running"),
);

const canEncode = computed(
  (): boolean => store.sources.length > 0 && !isRunning.value,
);

const canCancel = computed(
  (): boolean => isRunning.value && store.job.jobId !== null,
);

const canSave = computed(
  (): boolean =>
    isJobStatus(store.job.status, "done") &&
    store.job.jobId !== null &&
    store.job.result !== null,
);

const stageLabel = computed((): string | null => {
  const stage = store.job.stage;
  if (stage === null) {
    return null;
  }
  return STAGE_LABELS[stage];
});

const stderrTail = computed((): string =>
  store.job.stderrLines.slice(-STDERR_KEEP).join("\n"),
);

function resetJobForRun(jobId: string): void {
  const next: JobState = {
    status: "running",
    jobId,
    stage: "converting",
    stderrLines: [],
    result: null,
    error: null,
  };
  store.job = next;
}

async function onEncode(): Promise<void> {
  if (!canEncode.value) {
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
    store.job.error = null;
  } catch (err: unknown) {
    if (store.job.jobId !== jobId) {
      return;
    }
    const appError = toAppError(err);
    store.job.status = appError.kind === "cancelled" ? "idle" : "error";
    store.job.error = appError.kind === "cancelled" ? null : appError;
    store.job.result = null;
    if (appError.kind === "cancelled") {
      store.job.stage = null;
    }
  }
}

async function onCancel(): Promise<void> {
  const jobId = store.job.jobId;
  if (jobId === null || !isRunning.value) {
    return;
  }
  try {
    await call("cancel_encode", { job_id: jobId });
  } catch (err: unknown) {
    store.job.error = toAppError(err);
    store.job.status = "error";
  }
}

async function onSave(): Promise<void> {
  const jobId = store.job.jobId;
  if (jobId === null || !canSave.value) {
    return;
  }
  try {
    await call("save_encoded", { job_id: jobId });
  } catch (err: unknown) {
    store.job.error = toAppError(err);
    store.job.status = "error";
  }
}

const unlisteners: Array<() => void> = [];

onMounted(() => {
  void on("encode://progress", (payload) => {
    if (store.job.jobId === null || payload.jobId !== store.job.jobId) {
      return;
    }
    store.job.stage = payload.stage;
    if (payload.stderrLine !== null) {
      store.job.stderrLines.push(payload.stderrLine);
      if (store.job.stderrLines.length > STDERR_KEEP) {
        store.job.stderrLines.splice(
          0,
          store.job.stderrLines.length - STDERR_KEEP,
        );
      }
    }
  }).then((unlisten) => {
    unlisteners.push(unlisten);
  });
});

onUnmounted(() => {
  for (const unlisten of unlisteners) {
    unlisten();
  }
});
</script>

<template>
  <section class="panel shrink-0 overflow-hidden">
    <header class="border-b border-line px-3 py-2">
      <h2 class="m-0 text-sm font-semibold">
        Encode
      </h2>
    </header>

    <div class="flex flex-col gap-2 px-3 py-2">
      <div class="flex flex-wrap items-center gap-2">
        <Button
          variant="primary"
          :disabled="!canEncode"
          @click="onEncode"
        >
          Encode
        </Button>
        <Button
          :disabled="!canCancel"
          @click="onCancel"
        >
          Cancel
        </Button>
        <Button
          :disabled="!canSave"
          @click="onSave"
        >
          Save…
        </Button>
        <span
          v-if="stageLabel"
          class="text-xs text-ink-muted"
        >{{ stageLabel }}</span>
      </div>

      <ProgressBar
        v-if="isRunning"
        :value="null"
        :label="stageLabel ?? 'Working…'"
      />

      <pre
        v-if="stderrTail"
        class="m-0 max-h-28 overflow-auto rounded bg-surface-sunken p-2 font-mono text-xs whitespace-pre-wrap text-ink-muted"
      >{{ stderrTail }}</pre>

      <ErrorBox
        v-if="store.job.error"
        :error="store.job.error"
      />
    </div>
  </section>
</template>
