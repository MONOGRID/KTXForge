import { computed, ref } from "vue";

/** Where the next file drop should land. `null` = not over a drop panel. */
export type DropZone = "outputs" | "inputs" | "center";

/** True while a file drag is over the encoder webview. */
export const isFileDragActive = ref(false);

/** Updated while dragging over encoder panels. */
const activeDropZone = ref<DropZone | null>(null);

/**
 * Zone captured on the webview `drop` event. Prefer this when handling
 * `encoder://sources-added` — highlight state is cleared before that event arrives.
 */
const committedDropZone = ref<DropZone | null>(null);

export function setDropZone(zone: DropZone | null): void {
  activeDropZone.value = zone;
}

export function setFileDragActive(active: boolean): void {
  isFileDragActive.value = active;
  if (!active) {
    activeDropZone.value = null;
  }
}

/** Call from the webview drop event before clearing the highlight. */
export function commitDropZone(): void {
  committedDropZone.value = activeDropZone.value;
}

export function consumeCommittedDropZone(): DropZone | null {
  const zone = committedDropZone.value;
  committedDropZone.value = null;
  return zone;
}

export function isDropTarget(zone: DropZone): boolean {
  return isFileDragActive.value && activeDropZone.value === zone;
}

export const dropHintLabel = computed((): string => {
  switch (activeDropZone.value) {
    case "inputs":
      return "Add as inputs for the selected KTX";
    case "outputs":
      return "Create KTX outputs (one file → one output)";
    case "center":
      return "Choose action…";
    case null:
      return "Drop on Outputs, Inputs, or Preview";
  }
});
