<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { on } from "../../api/events";
import type { EncodeStage } from "../../bindings/EncodeStage";
import Button from "../../shared/components/Button.vue";
import ProgressBar from "../../shared/components/ProgressBar.vue";
import {
  canCancelEncode,
  canEncode,
  canSaveEncoded,
  cancelEncode,
  encode,
  saveEncoded,
} from "../actions";
import type { JobStatus } from "../store";
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

const encodeEnabled = computed((): boolean => canEncode());
const cancelEnabled = computed((): boolean => canCancelEncode());
const saveEnabled = computed((): boolean => canSaveEncoded());

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
          :disabled="!encodeEnabled"
          @click="encode"
        >
          Encode
        </Button>
        <Button
          :disabled="!cancelEnabled"
          @click="cancelEncode"
        >
          Cancel
        </Button>
        <Button
          :disabled="!saveEnabled"
          @click="saveEncoded"
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
    </div>
  </section>
</template>
