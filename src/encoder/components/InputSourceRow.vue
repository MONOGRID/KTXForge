<script setup lang="ts">
import { ChevronDown, ChevronUp, X } from "@lucide/vue";
import { computed } from "vue";
import type { SourceInfo } from "../../bindings/SourceInfo";
import HelpTooltip from "../../shared/components/HelpTooltip.vue";
import { moveInput, removeInput, store } from "../store";

const props = defineProps<{
  source: SourceInfo;
  index: number;
  lastIndex: number;
  thumbnailUrl: string | undefined;
  issueText: string;
}>();

const invalid = computed((): boolean => props.issueText.length > 0);

const canMoveUp = computed(
  (): boolean => props.index > 0 && store.queue === null,
);

const canMoveDown = computed(
  (): boolean => props.index < props.lastIndex && store.queue === null,
);
</script>

<template>
  <li class="mb-1 last:mb-0">
    <HelpTooltip
      class="flex w-full"
      :text="issueText"
      side="right"
    >
      <div
        class="flex w-full items-center gap-2 rounded-md border bg-surface-sunken px-2 py-1.5"
        :class="
          invalid ? 'border-danger/70 bg-danger/10' : 'border-transparent'
        "
      >
        <div class="h-10 w-10 shrink-0 overflow-hidden rounded bg-surface">
          <img
            v-if="thumbnailUrl"
            :src="thumbnailUrl"
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
            :disabled="!canMoveUp"
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
            :disabled="!canMoveDown"
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
      </div>
    </HelpTooltip>
  </li>
</template>
