<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Button from "../../shared/components/Button.vue";
import EnumSelect from "../../shared/components/EnumSelect.vue";
import type { EnumSelectOption } from "../../shared/enumSelect";
import {
  applyCenterDropAsOutputs,
  applyCenterDropAsPacked,
} from "../actions";
import { INPUT_ROLE_OPTIONS, type PackInputRole } from "../inputRole";
import { clearPendingCenterDrop, store } from "../store";

const packRole = ref<PackInputRole>("arrayLayers");

const visible = computed((): boolean => store.pendingCenterDrop !== null);

const fileCount = computed((): number => store.pendingCenterDrop?.sources.length ?? 0);

const roleOptions = computed((): EnumSelectOption[] =>
  INPUT_ROLE_OPTIONS.map((opt) => ({
    value: opt.value,
    label: opt.label,
    help: opt.help,
  })),
);

const packRoleLabel = computed((): string => {
  const found = INPUT_ROLE_OPTIONS.find((opt) => opt.value === packRole.value);
  return found?.label ?? "inputs";
});

watch(visible, (open) => {
  if (open) {
    packRole.value = "arrayLayers";
  }
});

function onRoleChange(value: string | null): void {
  if (value === null) {
    return;
  }
  packRole.value = value as PackInputRole;
}

function onCancel(): void {
  clearPendingCenterDrop();
}

function onSeparateOutputs(): void {
  void applyCenterDropAsOutputs();
}

function onPackOne(): void {
  void applyCenterDropAsPacked(packRole.value);
}
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-70 flex items-center justify-center bg-black/50 p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="drop-intent-title"
  >
    <section class="panel w-full max-w-md shadow-lg">
      <header class="border-b border-line px-4 py-3">
        <h2
          id="drop-intent-title"
          class="m-0 text-base font-semibold"
        >
          Drop {{ fileCount }} image{{ fileCount === 1 ? "" : "s" }}
        </h2>
      </header>
      <div class="flex flex-col gap-4 px-4 py-3">
        <p class="m-0 text-sm text-ink-muted">
          How should these files be used?
        </p>

        <button
          type="button"
          class="rounded border border-line bg-surface px-3 py-2.5 text-left transition-colors hover:border-primary hover:bg-primary/10"
          @click="onSeparateOutputs"
        >
          <div class="text-sm font-medium">
            Separate KTX outputs
          </div>
          <div class="hint mt-0.5">
            One output per image (batch encode)
          </div>
        </button>

        <div class="rounded border border-line bg-surface px-3 py-2.5">
          <div class="text-sm font-medium">
            Pack into one KTX
          </div>
          <div class="hint mt-0.5 mb-2">
            Multiple inputs for a single output — choose what the list means
          </div>
          <div class="field-row mb-3">
            <span class="label truncate">Kind</span>
            <EnumSelect
              class="min-w-0 flex-1"
              :model-value="packRole"
              :options="roleOptions"
              :empty-label="false"
              @update:model-value="onRoleChange"
            />
          </div>
          <Button
            variant="primary"
            class="w-full"
            @click="onPackOne"
          >
            Pack as {{ packRoleLabel }}
          </Button>
        </div>

        <div class="flex justify-end">
          <Button @click="onCancel">
            Cancel
          </Button>
        </div>
      </div>
    </section>
  </div>
</template>
