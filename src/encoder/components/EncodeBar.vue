<script setup lang="ts">
import { CirclePlay, CircleStop, Save, SaveAll } from "@lucide/vue";
import { computed, onMounted, onUnmounted } from "vue";
import { subscribe } from "../../api/events";
import type { EncodeStage } from "../../bindings/EncodeStage";
import Button from "../../shared/components/Button.vue";
import ProgressBar from "../../shared/components/ProgressBar.vue";
import {
  canCancelEncode,
  canEncode,
  canSaveAllEncoded,
  canSaveEncoded,
  cancelEncode,
  encode,
  saveAllEncoded,
  saveEncoded,
  slotByJobId,
} from "../actions";
import { selectedOutput, store } from "../store";

const STAGE_LABELS = {
  converting: "Converting…",
  encoding: "Encoding…",
  "writing-anim": "Writing animation…",
  previewing: "Building preview…",
  done: "Done",
} as const satisfies Record<EncodeStage, string>;

const encodeEnabled = computed((): boolean => canEncode());
const cancelEnabled = computed((): boolean => canCancelEncode());
const saveEnabled = computed((): boolean => canSaveEncoded());
const saveAllEnabled = computed((): boolean => canSaveAllEncoded());

const queueActive = computed((): boolean => store.queue !== null);

const selectedStageLabel = computed((): string | null => {
  const slot = selectedOutput.value;
  if (slot === null || slot.job.stage === null) {
    return null;
  }
  return STAGE_LABELS[slot.job.stage];
});

let stopProgress: (() => void) | undefined;

onMounted(() => {
  stopProgress = subscribe("encode://progress", (payload) => {
    const slot = slotByJobId(payload.jobId);
    if (slot === undefined) {
      return;
    }
    slot.job.stage = payload.stage;
  });
});

onUnmounted(() => {
  stopProgress?.();
});
</script>

<template>
  <div
    class="panel flex w-full flex-wrap items-center gap-3 px-2.5 py-1.5"
    role="group"
    aria-label="Encode"
  >
    <div class="min-w-0 flex-1">
      <ProgressBar
        v-if="queueActive"
        :value="
          store.queue !== null && store.queue.total > 0
            ? (store.queue.done / store.queue.total) * 100
            : null
        "
        :label="
          store.queue !== null
            ? `Done ${String(store.queue.done)} / ${String(store.queue.total)}`
            : 'Working…'
        "
      />
      <span
        v-else-if="selectedStageLabel"
        class="text-xs text-ink-muted"
      >{{ selectedStageLabel }}</span>
    </div>

    <div class="flex shrink-0 flex-wrap items-center justify-end gap-2">
      <Button
        variant="primary"
        :disabled="!encodeEnabled"
        @click="encode"
      >
        <CirclePlay
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        Encode all
      </Button>
      <Button
        :disabled="!cancelEnabled"
        @click="cancelEncode"
      >
        <CircleStop
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        Cancel queue
      </Button>
      <Button
        :disabled="!saveEnabled"
        @click="saveEncoded"
      >
        <Save
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        Save…
      </Button>
      <Button
        :disabled="!saveAllEnabled"
        @click="saveAllEncoded"
      >
        <SaveAll
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        Save all…
      </Button>
    </div>
  </div>
</template>
