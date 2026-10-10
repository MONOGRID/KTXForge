<script setup lang="ts">
import { ref, watch } from "vue";

const props = withDefaults(
  defineProps<{
    title: string;
    collapsible?: boolean;
    defaultOpen?: boolean;
  }>(),
  {
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
      <h2 class="m-0 text-sm font-semibold">
        {{ title }}
      </h2>
      <span
        v-if="collapsible"
        class="badge"
        aria-hidden="true"
      >{{ open ? "−" : "+" }}</span>
    </header>
    <div
      v-show="open"
      class="px-3 py-2"
    >
      <slot />
    </div>
  </section>
</template>
