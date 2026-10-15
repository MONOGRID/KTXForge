<script setup lang="ts">
import { Download, X } from "@lucide/vue";
import { computed, onMounted, onUnmounted, ref } from "vue";
import { call } from "../../api/commands";
import { subscribe } from "../../api/events";
import type { UpdateStatusPayload } from "../../bindings/UpdateStatusPayload";
import Button from "./Button.vue";
import ProgressBar from "./ProgressBar.vue";

const status = ref<UpdateStatusPayload | null>(null);
const retainedVersion = ref<string | null>(null);
const retainedNotes = ref<string | null>(null);
const installing = ref(false);
const dismissed = ref(false);

let stopStatus: (() => void) | undefined;

const visible = computed(
  (): boolean =>
    !dismissed.value &&
    status.value !== null &&
    status.value.state !== "installed",
);

const displayVersion = computed((): string | null => {
  const fromStatus = status.value?.version ?? null;
  return fromStatus ?? retainedVersion.value;
});

const displayNotes = computed((): string | null => {
  const fromStatus = status.value?.notes ?? null;
  return fromStatus ?? retainedNotes.value;
});

const progressPercent = computed((): number | null => {
  const current = status.value;
  if (current === null || current.state !== "downloading") {
    return null;
  }
  if (current.total <= 0) {
    return null;
  }
  return (current.downloaded / current.total) * 100;
});

const errorMessage = computed((): string | null => {
  const current = status.value;
  if (current === null || current.state !== "error") {
    return null;
  }
  return current.message ?? "Update failed";
});

/** An update was actually found; a failed check alone offers nothing to install. */
const canInstall = computed((): boolean => {
  const state = status.value?.state;
  return (
    retainedVersion.value !== null &&
    (state === "available" || state === "error")
  );
});

const installDisabled = computed((): boolean => {
  return installing.value || status.value?.state === "downloading";
});

function applyStatus(payload: UpdateStatusPayload): void {
  status.value = payload;
  retainedVersion.value = payload.version ?? retainedVersion.value;
  retainedNotes.value = payload.notes ?? retainedNotes.value;
  installing.value = payload.state === "downloading";
  dismissed.value = false;
}

onMounted(() => {
  stopStatus = subscribe("update://status", applyStatus);
});

onUnmounted(() => {
  stopStatus?.();
});

async function onInstall(): Promise<void> {
  installing.value = true;
  try {
    await call("install_update", {});
  } catch (err: unknown) {
    const message =
      err instanceof Error ? err.message : "Failed to install update";
    applyStatus({
      state: "error",
      version: retainedVersion.value,
      notes: retainedNotes.value,
      downloaded: 0,
      total: 0,
      message,
    });
  }
}
</script>

<template>
  <div
    v-if="visible"
    class="flex shrink-0 flex-col gap-2 border-b border-primary/30 bg-primary/10 px-4 py-2"
    role="status"
  >
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div class="min-w-0 flex-1">
        <div class="text-sm font-medium text-ink">
          <template v-if="displayVersion">
            Update {{ displayVersion }} available
          </template>
          <template v-else>
            Could not check for updates
          </template>
        </div>
        <p
          v-if="displayNotes"
          class="m-0 mt-1 max-h-24 overflow-auto text-xs text-ink-muted whitespace-pre-wrap"
        >
          {{ displayNotes }}
        </p>
      </div>
      <div class="flex shrink-0 items-center gap-2">
        <Button
          v-if="canInstall"
          variant="primary"
          type="button"
          :disabled="installDisabled"
          @click="onInstall"
        >
          <Download
            class="size-4 shrink-0"
            aria-hidden="true"
          />
          Install and restart
        </Button>
        <button
          type="button"
          class="btn shrink-0 px-1.5 py-0.5"
          aria-label="Dismiss"
          :disabled="status?.state === 'downloading'"
          @click="dismissed = true"
        >
          <X
            class="size-3.5"
            aria-hidden="true"
          />
        </button>
      </div>
    </div>

    <ProgressBar
      v-if="status?.state === 'downloading'"
      :value="progressPercent"
      label="Downloading update…"
    />

    <p
      v-if="errorMessage !== null"
      class="m-0 text-sm text-danger"
      role="alert"
    >
      {{ errorMessage }}
    </p>
  </div>
</template>
