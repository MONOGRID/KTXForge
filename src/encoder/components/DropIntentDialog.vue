<script setup lang="ts">
import { Combine, ImageUp, PackagePlus, Split, X } from "@lucide/vue";
import { computed, ref, watch } from "vue";
import Button from "../../shared/components/Button.vue";
import EnumSelect from "../../shared/components/EnumSelect.vue";
import type { EnumSelectOption } from "../../shared/enumSelect";
import {
  applyCenterDropAsOutputs,
  applyCenterDropAsPacked,
} from "../actions";
import {
  INPUT_ROLE_OPTIONS,
  parsePackRole,
  type PackInputRole,
} from "../inputRole";
import { INPUT_ROLE_ICONS } from "../inputRoleIcons";
import { clearPendingCenterDrop, store } from "../store";

const packRole = ref<PackInputRole>("arrayLayers");

const visible = computed((): boolean => store.pendingCenterDrop !== null);

const fileCount = computed((): number => store.pendingCenterDrop?.sources.length ?? 0);

const roleOptions = computed((): EnumSelectOption[] =>
  INPUT_ROLE_OPTIONS.map((opt) => ({
    value: opt.value,
    label: opt.label,
    help: opt.help,
    icon: INPUT_ROLE_ICONS[opt.value],
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
  const role = value === null ? null : parsePackRole(value);
  if (role !== null) {
    packRole.value = role;
  }
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
          class="m-0 flex items-center gap-2 text-base font-semibold"
        >
          <ImageUp
            class="size-5 shrink-0 text-ink-muted"
            aria-hidden="true"
          />
          Drop {{ fileCount }} image{{ fileCount === 1 ? "" : "s" }}
        </h2>
      </header>
      <div class="flex flex-col gap-4 px-4 py-3">
        <p class="m-0 text-sm text-ink-muted">
          How should these files be used?
        </p>

        <button
          type="button"
          class="flex gap-3 rounded-lg border border-transparent bg-surface-sunken px-3 py-2.5 text-left shadow-sm transition-colors hover:bg-primary/10 hover:ring-2 hover:ring-inset hover:ring-primary/40"
          @click="onSeparateOutputs"
        >
          <Split
            class="mt-0.5 size-4 shrink-0 text-ink-muted"
            aria-hidden="true"
          />
          <div class="min-w-0">
            <div class="text-sm font-medium">
              Separate KTX outputs
            </div>
            <div class="hint mt-0.5">
              One output per image (batch encode)
            </div>
          </div>
        </button>

        <div class="flex gap-3 rounded-lg border border-transparent bg-surface-sunken px-3 py-2.5 shadow-sm">
          <Combine
            class="mt-0.5 size-4 shrink-0 text-ink-muted"
            aria-hidden="true"
          />
          <div class="min-w-0 flex-1">
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
              <PackagePlus
                class="size-4 shrink-0"
                aria-hidden="true"
              />
              Pack as {{ packRoleLabel }}
            </Button>
          </div>
        </div>

        <div class="flex justify-end">
          <Button @click="onCancel">
            <X
              class="size-4 shrink-0"
              aria-hidden="true"
            />
            Cancel
          </Button>
        </div>
      </div>
    </section>
  </div>
</template>
