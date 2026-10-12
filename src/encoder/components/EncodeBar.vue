<script setup lang="ts">
import { CirclePlay, CircleStop, Cpu, Save, SaveAll } from "@lucide/vue";
import { computed, onMounted, onUnmounted } from "vue";
import { on } from "../../api/events";
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

const unlisteners: Array<() => void> = [];

onMounted(() => {
  void on("encode://progress", (payload) => {
    const slot = slotByJobId(payload.jobId);
    if (slot === undefined) {
      return;
    }
    slot.job.stage = payload.stage;
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
      <h2 class="m-0 flex items-center gap-1.5 text-sm font-semibold">
        <Cpu
          class="size-4 shrink-0 text-ink-muted"
          aria-hidden="true"
        />
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
        <span
          v-if="!queueActive && selectedStageLabel"
          class="text-xs text-ink-muted"
        >{{ selectedStageLabel }}</span>
      </div>

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
    </div>
  </section>
</template>
