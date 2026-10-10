<script setup lang="ts">
// fallow-ignore-file unused-file
// consumed by T5.4

withDefaults(
  defineProps<{
    variant?: "default" | "primary";
    disabled?: boolean;
    type?: "button" | "submit" | "reset";
  }>(),
  {
    variant: "default",
    disabled: false,
    type: "button",
  },
);

defineEmits<{
  click: [event: MouseEvent];
}>();
</script>

<template>
  <button
    :type="type"
    class="btn"
    :class="{ 'btn-primary': variant === 'primary' }"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>
