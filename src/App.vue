<script setup lang="ts">
import { invoke, isTauri } from "@tauri-apps/api/core";
import { ref } from "vue";

const name = ref("");
const message = ref("Type a name and greet from Rust.");

async function greet() {
  const trimmed = name.value.trim();
  if (!trimmed) {
    message.value = "Enter a name first.";
    return;
  }

  if (!isTauri()) {
    message.value = `Hello, ${trimmed}. Rust greet runs in the Tauri window.`;
    return;
  }

  message.value = await invoke<string>("greet", { name: trimmed });
}
</script>

<template>
  <main class="box-border grid min-h-screen place-items-center bg-[#f6f3ee] p-8 text-stone-900 font-['Segoe_UI',sans-serif]">
    <section class="w-full max-w-md">
      <p class="m-0 text-xs text-[#0f6e56] font-650 tracking-[0.14em] uppercase">
        Desktop
      </p>
      <h1 class="m-0 mt-1 text-5xl font-650 -tracking-[0.04em]">
        Hello
      </h1>
      <p class="m-0 mt-3 text-stone-600">
        A Vue page on this Tauri app.
      </p>
      <form
        class="mt-6 flex gap-2"
        @submit.prevent="greet"
      >
        <input
          v-model="name"
          class="min-w-0 flex-1 border border-stone-300 rounded-lg bg-white px-[0.85rem] py-[0.65rem] text-inherit [font:inherit] border-solid"
          type="text"
          name="name"
          placeholder="Your name"
          autocomplete="name"
        >
        <button
          class="cursor-pointer border border-[#0f6e56] rounded-lg bg-[#0f6e56] px-[0.85rem] py-[0.65rem] text-white [font:inherit] border-solid"
          type="submit"
        >
          Greet
        </button>
      </form>
      <p
        class="m-0 mt-4 min-h-6"
        role="status"
      >
        {{ message }}
      </p>
    </section>
  </main>
</template>
