<script setup lang="ts">
import { Boxes, ImagePlus, Plus, X } from "@lucide/vue";
import { computed } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { addInputsForOutput, openSourcesAsOutputs } from "../actions";
import { isDropTarget, setDropZone } from "../dropTarget";
import {
  outputLabel,
  removeOutput,
  selectOutput,
  store,
  type OutputSlot,
} from "../store";

const rows = computed((): OutputSlot[] => store.outputs);

const highlighted = computed((): boolean => isDropTarget("outputs"));

function statusLabel(slot: OutputSlot): string {
  switch (slot.job.status) {
    case "idle":
      return "";
    case "queued":
      return "queued";
    case "running":
      return "encoding";
    case "done":
      return "done";
    case "error":
      return "error";
  }
}

function onAdd(): void {
  void openSourcesAsOutputs();
}

function onAddInputs(slot: OutputSlot): void {
  void addInputsForOutput(slot.id);
}

function onPointerEnter(): void {
  setDropZone("outputs");
}

function canRemove(slot: OutputSlot): boolean {
  return (
    store.queue === null &&
    slot.job.status !== "running" &&
    slot.job.status !== "queued"
  );
}

function canAddInputs(slot: OutputSlot): boolean {
  return (
    store.queue === null &&
    slot.job.status !== "running" &&
    slot.job.status !== "queued"
  );
}
</script>

<template>
  <section
    class="panel flex h-full min-h-0 flex-col overflow-hidden transition-colors"
    :class="
      highlighted ? 'ring-2 ring-inset ring-primary/50 bg-primary/10' : ''
    "
    data-drop-zone="outputs"
    @pointerenter="onPointerEnter"
  >
    <header class="flex items-center justify-between gap-2 border-b border-line px-3 py-2">
      <h2 class="m-0 flex min-w-0 items-center gap-1.5 text-sm font-semibold">
        <Boxes
          class="size-4 shrink-0 text-ink-muted"
          aria-hidden="true"
        />
        KTX outputs
      </h2>
      <button
        type="button"
        class="btn"
        :disabled="store.queue !== null"
        @click="onAdd"
      >
        <Plus
          class="size-4 shrink-0"
          aria-hidden="true"
        />
        Add…
      </button>
    </header>

    <ScrollArea
      v-if="rows.length > 0"
      class="min-h-0 flex-1"
    >
      <ul class="m-0 list-none p-2 pr-3">
        <li
          v-for="slot in rows"
          :key="slot.id"
          class="mb-1 flex items-center gap-2 rounded-md border px-2 py-1.5 last:mb-0"
          :class="
            slot.id === store.selectedId
              ? 'border-transparent bg-primary/15 ring-2 ring-inset ring-primary/45'
              : 'border-transparent bg-surface-sunken'
          "
        >
          <button
            type="button"
            class="min-w-0 flex-1 border-0 bg-transparent p-0 text-left"
            @click="selectOutput(slot.id)"
          >
            <div class="truncate text-sm font-medium">
              {{ outputLabel(slot) }}
            </div>
            <div class="mt-0.5 text-xs text-ink-muted">
              {{ slot.inputs.length }}
              input{{ slot.inputs.length === 1 ? "" : "s" }}
              <span v-if="statusLabel(slot)"> · {{ statusLabel(slot) }}</span>
            </div>
          </button>
          <button
            type="button"
            class="btn shrink-0 px-1.5 py-0.5"
            aria-label="Add inputs for this output"
            title="Add inputs"
            :disabled="!canAddInputs(slot)"
            @click="onAddInputs(slot)"
          >
            <ImagePlus
              class="size-3.5"
              aria-hidden="true"
            />
          </button>
          <button
            type="button"
            class="btn shrink-0 px-1.5 py-0.5"
            aria-label="Remove output"
            :disabled="!canRemove(slot)"
            @click="removeOutput(slot.id)"
          >
            <X
              class="size-3.5"
              aria-hidden="true"
            />
          </button>
        </li>
      </ul>
    </ScrollArea>

    <p
      v-else
      class="m-0 flex flex-1 items-center justify-center px-4 text-center text-sm text-ink-muted"
    >
      Drop images here to create KTX outputs (one file → one output)
    </p>
  </section>
</template>
