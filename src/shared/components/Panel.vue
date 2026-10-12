<script setup lang="ts">
import type { Component } from "vue";
import { ChevronDown, ChevronRight } from "@lucide/vue";
import { ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    title: string;
    /** Leading Lucide (or other) icon beside the title. */
    icon?: Component;
    collapsible?: boolean;
    defaultOpen?: boolean;
  }>(),
  {
    icon: undefined,
    collapsible: true,
    defaultOpen: true,
  },
);

const open = ref(props.defaultOpen);

watch(
  () => props.defaultOpen,
  (value) => {
    open.value = value;
  },
);

function toggle(): void {
  if (props.collapsible) {
    open.value = !open.value;
  }
}
</script>

<template>
  <section class="panel overflow-hidden">
    <header
      class="flex items-center justify-between gap-2 border-b border-line px-3 py-2"
      :class="{ 'cursor-pointer select-none': collapsible }"
      @click="toggle"
    >
      <h2 class="m-0 flex min-w-0 items-center gap-1.5 text-sm font-semibold">
        <component
          :is="icon"
          v-if="icon"
          class="size-4 shrink-0 text-ink-muted"
          aria-hidden="true"
        />
        <span class="min-w-0 truncate">{{ title }}</span>
      </h2>
      <component
        :is="open ? ChevronDown : ChevronRight"
        v-if="collapsible"
        class="size-4 shrink-0 text-ink-muted"
        aria-hidden="true"
      />
    </header>
    <div
      v-show="open"
      class="px-3 py-2"
    >
      <slot />
    </div>
  </section>
</template>
