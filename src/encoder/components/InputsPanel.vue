<script setup lang="ts">
import { ChevronDown, ChevronUp, Images, Plus, X } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { computed, onUnmounted, reactive, watch } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { call } from "../../api/commands";
import type { SourceInfo } from "../../bindings/SourceInfo";
import EnumSelect from "../../shared/components/EnumSelect.vue";
import type { EnumSelectOption } from "../../shared/enumSelect";
import { openSourcesAsInputs } from "../actions";
import { isDropTarget, setDropZone } from "../dropTarget";
import { INPUT_ROLE_OPTIONS, isPackRole, parsePackRole } from "../inputRole";
import { INPUT_ROLE_ICONS } from "../inputRoleIcons";
import {
  moveInput,
  removeInput,
  selectedOutput,
  selectedRole,
  setInputRole,
  store,
} from "../store";

const thumbnails = reactive(new Map<string, string>());
const notice = reactive<{ message: string | null }>({ message: null });

const { start: startNoticeTimer, stop: stopNoticeTimer } = useTimeoutFn(
  () => {
    notice.message = null;
  },
  6000,
  { immediate: false },
);

function showNotice(message: string): void {
  notice.message = message;
  stopNoticeTimer();
  startNoticeTimer();
}

function revokeThumbnail(path: string): void {
  const url = thumbnails.get(path);
  if (url !== undefined) {
    URL.revokeObjectURL(url);
    thumbnails.delete(path);
  }
}

async function loadThumbnail(source: SourceInfo): Promise<void> {
  if (thumbnails.has(source.path)) {
    return;
  }
  try {
    const bytes = await call("source_thumbnail", { path: source.path });
    const url = URL.createObjectURL(new Blob([bytes], { type: "image/png" }));
    thumbnails.set(source.path, url);
  } catch {
    // Leave the slot empty.
  }
}

const inputs = computed((): SourceInfo[] => selectedOutput.value?.inputs ?? []);

const highlighted = computed((): boolean => isDropTarget("inputs"));

const roleOptions = computed((): EnumSelectOption[] =>
  INPUT_ROLE_OPTIONS.map((opt) => ({
    value: opt.value,
    label: opt.label,
    help: opt.help,
    icon: INPUT_ROLE_ICONS[opt.value],
  })),
);

/** Pack roles only — Base 2D is implicit when the Inputs panel is hidden. */
const roleModel = computed((): string =>
  isPackRole(selectedRole.value) ? selectedRole.value : "arrayLayers",
);

watch(
  () => inputs.value.map((s) => s.path),
  (paths, prev) => {
    const next = new Set(paths);
    for (const path of prev ?? []) {
      if (!next.has(path)) {
        revokeThumbnail(path);
      }
    }
    for (const source of inputs.value) {
      void loadThumbnail(source);
    }
  },
  { immediate: true },
);

onUnmounted(() => {
  for (const path of [...thumbnails.keys()]) {
    revokeThumbnail(path);
  }
  stopNoticeTimer();
});

function onAdd(): void {
  void openSourcesAsInputs();
}

function onRoleChange(value: string | null): void {
  const role = value === null ? null : parsePackRole(value);
  if (role !== null) {
    setInputRole(role);
  }
}

function onPointerEnter(): void {
  setDropZone("inputs");
}

defineExpose({ showNotice });
</script>

<template>
  <section
    class="panel flex h-full min-h-0 flex-col overflow-hidden transition-colors"
    :class="
      highlighted ? 'ring-2 ring-inset ring-primary/50 bg-primary/10' : ''
    "
    data-drop-zone="inputs"
    @pointerenter="onPointerEnter"
  >
    <header class="flex flex-col gap-2 border-b border-line px-3 py-2">
      <div class="flex items-center justify-between gap-2">
        <h2 class="m-0 flex min-w-0 items-center gap-1.5 text-sm font-semibold">
          <Images
            class="size-4 shrink-0 text-ink-muted"
            aria-hidden="true"
          />
          Inputs
        </h2>
        <button
          type="button"
          class="btn"
          :disabled="selectedOutput === null || store.queue !== null"
          @click="onAdd"
        >
          <Plus
            class="size-4 shrink-0"
            aria-hidden="true"
          />
          Add…
        </button>
      </div>
      <div class="field-row">
        <span class="label truncate">Role</span>
        <EnumSelect
          class="min-w-0 flex-1"
          :model-value="roleModel"
          :options="roleOptions"
          :empty-label="false"
          :disabled="store.queue !== null"
          @update:model-value="onRoleChange"
        />
      </div>
    </header>

    <p
      v-if="notice.message"
      class="m-0 border-b border-danger/40 bg-danger/10 px-3 py-2 text-xs text-danger"
      role="status"
    >
      {{ notice.message }}
    </p>

    <p
      v-if="selectedOutput === null"
      class="m-0 flex flex-1 items-center justify-center px-4 text-center text-sm text-ink-muted"
    >
      Select a KTX output to edit its inputs
    </p>

    <ScrollArea
      v-else-if="inputs.length > 0"
      class="min-h-0 flex-1"
    >
      <ul class="m-0 list-none p-2 pr-3">
        <li
          v-for="(source, index) in inputs"
          :key="`${source.path}:${String(index)}`"
          class="mb-1 flex items-center gap-2 rounded-md border border-transparent bg-surface-sunken px-2 py-1.5 last:mb-0"
        >
          <div class="h-10 w-10 shrink-0 overflow-hidden rounded bg-surface">
            <img
              v-if="thumbnails.get(source.path)"
              :src="thumbnails.get(source.path)"
              :alt="source.name"
              class="h-full w-full object-contain"
            >
          </div>
          <div class="min-w-0 flex-1">
            <div class="truncate text-sm font-medium">
              {{ source.name }}
            </div>
            <div class="hint">
              {{ source.width }}×{{ source.height }} · {{ source.kind }}
            </div>
          </div>
          <div class="flex shrink-0 flex-col gap-0.5">
            <button
              type="button"
              class="btn px-1.5 py-0.5"
              :disabled="index === 0 || store.queue !== null"
              aria-label="Move up"
              @click="moveInput(index, -1)"
            >
              <ChevronUp
                class="size-3.5"
                aria-hidden="true"
              />
            </button>
            <button
              type="button"
              class="btn px-1.5 py-0.5"
              :disabled="index === inputs.length - 1 || store.queue !== null"
              aria-label="Move down"
              @click="moveInput(index, 1)"
            >
              <ChevronDown
                class="size-3.5"
                aria-hidden="true"
              />
            </button>
          </div>
          <button
            type="button"
            class="btn shrink-0 px-1.5 py-0.5"
            aria-label="Remove"
            :disabled="store.queue !== null"
            @click="removeInput(index)"
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
      Drop images here to add inputs for the selected output
    </p>
  </section>
</template>
