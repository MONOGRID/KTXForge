<script setup lang="ts">
import { Images, Plus } from "@lucide/vue";
import { useTimeoutFn } from "@vueuse/core";
import { computed, onUnmounted, reactive, watch } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import { call } from "../../api/commands";
import type { SourceInfo } from "../../bindings/SourceInfo";
import EnumSelect from "../../shared/components/EnumSelect.vue";
import HelpTooltip from "../../shared/components/HelpTooltip.vue";
import type { EnumSelectOption } from "../../shared/enumSelect";
import { openSourcesAsInputs } from "../actions";
import { isDropTarget, setDropZone } from "../dropTarget";
import { INPUT_ROLE_OPTIONS, isPackRole, parsePackRole } from "../inputRole";
import { INPUT_ROLE_ICONS } from "../inputRoleIcons";
import InputSourceRow from "./InputSourceRow.vue";
import {
  selectedInputReport,
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

const panelError = computed((): string | null => selectedInputReport.value.panel);

const panelClass = computed((): string => {
  if (highlighted.value) {
    return "ring-2 ring-inset ring-primary/50 bg-primary/10";
  }
  if (panelError.value !== null) {
    return "ring-2 ring-inset ring-danger/60 bg-danger/5";
  }
  return "";
});

/** Joined messages per input index for row chrome + tooltips. */
const rowIssues = computed((): Map<number, string> => {
  const map = new Map<number, string>();
  for (const issue of selectedInputReport.value.inputs) {
    const prev = map.get(issue.index);
    map.set(
      issue.index,
      prev === undefined ? issue.message : `${prev}; ${issue.message}`,
    );
  }
  return map;
});

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

const lastIndex = computed((): number => Math.max(0, inputs.value.length - 1));

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

function rowIssue(index: number): string {
  return rowIssues.value.get(index) ?? "";
}

defineExpose({ showNotice });
</script>

<template>
  <HelpTooltip
    class="flex h-full min-h-0 w-full flex-col"
    :text="panelError ?? ''"
    side="right"
  >
    <section
      class="panel flex h-full min-h-0 w-full flex-col overflow-hidden transition-colors"
      :class="panelClass"
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
          <InputSourceRow
            v-for="(source, index) in inputs"
            :key="`${source.path}:${String(index)}`"
            :source="source"
            :index="index"
            :last-index="lastIndex"
            :thumbnail-url="thumbnails.get(source.path)"
            :issue-text="rowIssue(index)"
          />
        </ul>
      </ScrollArea>

      <p
        v-else
        class="m-0 flex flex-1 items-center justify-center px-4 text-center text-sm text-ink-muted"
      >
        Drop images here to add inputs for the selected output
      </p>
    </section>
  </HelpTooltip>
</template>
