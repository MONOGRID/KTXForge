<script setup lang="ts">
import { onKeyStroke } from "@vueuse/core";
import { Check, X } from "@lucide/vue";
import { computed } from "vue";
import { ScrollArea } from "@/components/ui/scroll-area";
import Button from "../../shared/components/Button.vue";

const props = defineProps<{
  open: boolean;
  files: readonly string[];
  summary: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const countLabel = computed((): string => {
  const n = props.files.length;
  return n === 1 ? "1 file written" : `${String(n)} files written`;
});

function onClose(): void {
  emit("close");
}

onKeyStroke("Escape", (event) => {
  if (!props.open) {
    return;
  }
  event.preventDefault();
  onClose();
});
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-60 flex items-center justify-center bg-black/50 p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="export-result-title"
    @click.self="onClose"
  >
    <section class="panel flex max-h-[min(36rem,90vh)] w-full max-w-lg flex-col shadow-lg">
      <header class="shrink-0 border-b border-line px-4 py-3">
        <h2
          id="export-result-title"
          class="m-0 flex items-center gap-2 text-base font-semibold"
        >
          <Check
            class="size-5 shrink-0 text-primary"
            aria-hidden="true"
          />
          Export complete
        </h2>
      </header>

      <div class="flex min-h-0 flex-1 flex-col gap-3 px-4 py-3">
        <div class="shrink-0">
          <p class="m-0 text-sm">
            {{ countLabel }}
          </p>
          <p class="m-0 mt-1 text-xs text-ink-muted">
            {{ summary }}
          </p>
        </div>

        <ScrollArea class="min-h-0 max-h-64 flex-1 rounded-md border border-line/50 bg-surface-sunken">
          <ul class="m-0 list-none p-2 font-mono text-xs text-ink-muted">
            <li
              v-for="file in files"
              :key="file"
              class="break-all border-b border-line/30 px-1.5 py-1 last:border-b-0"
              :title="file"
            >
              {{ file }}
            </li>
          </ul>
        </ScrollArea>

        <div class="flex shrink-0 justify-end">
          <Button
            variant="primary"
            @click="onClose"
          >
            <X
              class="size-4 shrink-0"
              aria-hidden="true"
            />
            Close
          </Button>
        </div>
      </div>
    </section>
  </div>
</template>
