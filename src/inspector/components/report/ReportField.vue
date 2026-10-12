<script setup lang="ts">
defineProps<{
  label: string;
  value: string;
}>();
</script>

<template>
  <div class="field-row">
    <div class="label">
      {{ label }}
    </div>
    <div class="min-w-0 break-words font-mono text-xs text-ink">
      {{ value }}
    </div>
  </div>
</template>
