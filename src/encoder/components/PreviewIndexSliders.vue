<script setup lang="ts">
defineProps<{
  showPackSlider: boolean;
  showGeneratedMipSlider: boolean;
  packSliderLabel: string;
  generatedMipLabel: string;
  packMax: number;
  generatedMipMax: number;
  previewIndex: number;
  encodedMip: number;
  packAxisIsMip: boolean;
}>();

const emit = defineEmits<{
  packInput: [event: Event];
  generatedMipInput: [event: Event];
}>();
</script>

<template>
  <div
    v-if="showPackSlider || showGeneratedMipSlider"
    class="flex shrink-0 flex-col gap-1"
  >
    <div
      v-if="showPackSlider"
      class="flex items-center gap-2"
    >
      <label
        class="shrink-0 whitespace-nowrap text-xs text-ink-muted"
        for="preview-pack"
      >{{ packSliderLabel }}</label>
      <input
        id="preview-pack"
        type="range"
        class="h-4 min-w-0 flex-1 accent-brand"
        min="0"
        :max="packMax"
        step="1"
        :value="previewIndex"
        :aria-valuemin="0"
        :aria-valuemax="packMax"
        :aria-valuenow="previewIndex"
        @input="emit('packInput', $event)"
      >
      <span class="w-7 shrink-0 text-right font-mono text-[10px] tabular-nums text-ink-faint">
        {{ packAxisIsMip ? `L${String(packMax)}` : String(packMax) }}
      </span>
    </div>

    <div
      v-if="showGeneratedMipSlider"
      class="flex items-center gap-2"
    >
      <label
        class="shrink-0 whitespace-nowrap text-xs text-ink-muted"
        for="preview-gen-mip"
      >{{ generatedMipLabel }}</label>
      <input
        id="preview-gen-mip"
        type="range"
        class="h-4 min-w-0 flex-1 accent-brand"
        min="0"
        :max="generatedMipMax"
        step="1"
        :value="encodedMip"
        :aria-valuemin="0"
        :aria-valuemax="generatedMipMax"
        :aria-valuenow="encodedMip"
        @input="emit('generatedMipInput', $event)"
      >
      <span class="w-7 shrink-0 text-right font-mono text-[10px] tabular-nums text-ink-faint">
        L{{ generatedMipMax }}
      </span>
    </div>
  </div>
</template>
