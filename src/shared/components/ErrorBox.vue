<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4
import { computed } from "vue";
import { useClipboard } from "@vueuse/core";
import type { AppError } from "../../bindings/AppError";
import Button from "./Button.vue";

const props = defineProps<{
  error: AppError;
}>();

const { copy, copied } = useClipboard();

const exitCode = computed((): number | undefined => {
  if (props.error.kind === "ktx") {
    return props.error.exitCode;
  }
  return undefined;
});

const exitMeaning = computed((): string | undefined => {
  if (props.error.kind === "ktx") {
    return props.error.exitMeaning;
  }
  return undefined;
});

const stderr = computed((): string | undefined => {
  if (props.error.kind === "ktx") {
    return props.error.stderr;
  }
  return undefined;
});

const copyText = computed((): string => {
  const parts = [`[${props.error.kind}] ${props.error.message}`];
  if (exitCode.value !== undefined) {
    parts.push(`exit ${String(exitCode.value)}: ${exitMeaning.value ?? ""}`);
  }
  if (stderr.value) {
    parts.push(stderr.value);
  }
  return parts.join("\n");
});

async function onCopy(): Promise<void> {
  await copy(copyText.value);
}
</script>

<template>
  <div class="rounded border border-danger/40 bg-danger/10 p-3 text-sm text-ink">
    <div class="flex items-start justify-between gap-2">
      <div>
        <div class="font-medium text-danger">
          {{ error.message }}
        </div>
        <div
          v-if="exitCode !== undefined"
          class="hint"
        >
          Exit {{ exitCode }} — {{ exitMeaning }}
        </div>
      </div>
      <Button
        type="button"
        @click="onCopy"
      >
        {{ copied ? "Copied" : "Copy" }}
      </Button>
    </div>
    <pre
      v-if="stderr"
      class="mt-2 max-h-40 overflow-auto rounded bg-surface-sunken p-2 font-mono text-xs whitespace-pre-wrap"
    >{{ stderr }}</pre>
  </div>
</template>
