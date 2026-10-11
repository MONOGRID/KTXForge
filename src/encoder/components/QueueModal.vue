<script setup lang="ts">
import { computed } from "vue";
import Button from "../../shared/components/Button.vue";
import ProgressBar from "../../shared/components/ProgressBar.vue";
import { cancelEncode } from "../actions";
import { outputLabel, store } from "../store";

const visible = computed((): boolean => store.queue !== null);

const done = computed((): number => store.queue?.done ?? 0);
const total = computed((): number => store.queue?.total ?? 0);

const pct = computed((): number | null => {
  if (total.value === 0) {
    return null;
  }
  return (done.value / total.value) * 100;
});

const runningNames = computed((): string => {
  const names = store.outputs
    .filter((s) => s.job.status === "running")
    .map((s) => outputLabel(s));
  if (names.length === 0) {
    return "Starting…";
  }
  return names.join(", ");
});

function onCancel(): void {
  void cancelEncode();
}
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-60 flex items-center justify-center bg-black/50 p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="encode-queue-title"
  >
    <section class="panel w-full max-w-md shadow-lg">
      <header class="border-b border-line px-4 py-3">
        <h2
          id="encode-queue-title"
          class="m-0 text-base font-semibold"
        >
          Encoding queue
        </h2>
      </header>
      <div class="flex flex-col gap-3 px-4 py-3">
        <p class="m-0 text-sm">
          Done {{ done }} / {{ total }}
        </p>
        <ProgressBar
          :value="pct"
          :label="runningNames"
        />
        <p class="m-0 text-xs text-ink-muted">
          {{ runningNames }}
        </p>
        <div class="flex justify-end">
          <Button @click="onCancel">
            Cancel queue
          </Button>
        </div>
      </div>
    </section>
  </div>
</template>
