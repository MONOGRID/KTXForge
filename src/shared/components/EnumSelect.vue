<script setup lang="ts">
import { computed, inject } from "vue";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";
import type { EnumSelectGroup, EnumSelectOption } from "../enumSelect";
import HelpTooltip from "./HelpTooltip.vue";
import { optionControlIdKey } from "./optionControlId";

const EMPTY_VALUE = "__empty__";

const props = withDefaults(
  defineProps<{
    modelValue: string | null;
    options?: readonly EnumSelectOption[];
    groups?: readonly EnumSelectGroup[];
    /** Label for the empty/null choice. Pass `false` to omit it. */
    emptyLabel?: string | false;
    emptyHelp?: string;
    id?: string;
    disabled?: boolean;
    class?: string;
  }>(),
  {
    options: () => [],
    groups: () => [],
    emptyLabel: "Default",
    emptyHelp: undefined,
    id: undefined,
    disabled: false,
    class: undefined,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string | null];
}>();

const injectedId = inject(optionControlIdKey, undefined);

const controlId = computed((): string | undefined => props.id ?? injectedId);

const selectValue = computed((): string => {
  if (props.modelValue === null || props.modelValue === "") {
    return EMPTY_VALUE;
  }
  return props.modelValue;
});

const placeholder = computed((): string =>
  props.emptyLabel === false ? "Select" : props.emptyLabel,
);

function onUpdate(value: unknown): void {
  if (typeof value !== "string") {
    return;
  }
  emit("update:modelValue", value === EMPTY_VALUE ? null : value);
}
</script>

<template>
  <Select
    :model-value="selectValue"
    :disabled="disabled"
    @update:model-value="onUpdate"
  >
    <SelectTrigger
      :id="controlId"
      :class="cn('w-full font-mono', props.class)"
    >
      <SelectValue :placeholder="placeholder" />
    </SelectTrigger>
    <SelectContent>
      <SelectItem
        v-if="emptyLabel !== false"
        :value="EMPTY_VALUE"
      >
        <HelpTooltip
          :text="emptyHelp ?? ''"
          side="right"
          class="w-full"
        >
          <span class="block w-full truncate">{{ emptyLabel }}</span>
        </HelpTooltip>
      </SelectItem>
      <SelectItem
        v-for="option in options"
        :key="option.value"
        :value="option.value"
      >
        <HelpTooltip
          :text="option.help ?? ''"
          side="right"
          class="w-full"
        >
          <span class="flex w-full min-w-0 items-center gap-1.5">
            <component
              :is="option.icon"
              v-if="option.icon"
              class="size-3 shrink-0"
              aria-hidden="true"
            />
            <span class="min-w-0 truncate">{{ option.label }}</span>
          </span>
        </HelpTooltip>
      </SelectItem>
      <SelectGroup
        v-for="group in groups"
        :key="group.label"
      >
        <SelectLabel>{{ group.label }}</SelectLabel>
        <SelectItem
          v-for="option in group.options"
          :key="option.value"
          :value="option.value"
        >
          <HelpTooltip
            :text="option.help ?? ''"
            side="right"
            class="w-full"
          >
            <span class="flex w-full min-w-0 items-center gap-1.5">
              <component
                :is="option.icon"
                v-if="option.icon"
                class="size-3 shrink-0"
                aria-hidden="true"
              />
              <span class="min-w-0 truncate">{{ option.label }}</span>
            </span>
          </HelpTooltip>
        </SelectItem>
      </SelectGroup>
    </SelectContent>
  </Select>
</template>
