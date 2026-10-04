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
  <main class="box-border grid min-h-screen place-items-center p-8">
    <section class="home w-full max-w-md">
      <p class="home-kicker">
        Desktop
      </p>
      <h1 class="home-title">
        Hello
      </h1>
      <p class="home-lede">
        A Vue page on this Tauri app.
      </p>
      <form
        class="mt-6 flex gap-2"
        @submit.prevent="greet"
      >
        <input
          v-model="name"
          class="home-field min-w-0 flex-1"
          type="text"
          name="name"
          placeholder="Your name"
          autocomplete="name"
        >
        <button
          class="home-button"
          type="submit"
        >
          Greet
        </button>
      </form>
      <p
        class="home-message"
        role="status"
      >
        {{ message }}
      </p>
    </section>
  </main>
</template>

<style lang="scss">
$ink: #1c1917;
$paper: #f6f3ee;
$line: #d6d3d1;
$accent: #0f6e56;

:root {
  color: $ink;
  background: $paper;
  font-family: "Segoe UI", sans-serif;
}

body {
  margin: 0;
}

.home-kicker {
  margin: 0;
  color: $accent;
  font-size: 0.75rem;
  font-weight: 650;
  letter-spacing: 0.14em;
  text-transform: uppercase;
}

.home-title {
  margin: 0.25rem 0 0;
  font-size: 3rem;
  font-weight: 650;
  letter-spacing: -0.04em;
  line-height: 1;
}

.home-lede {
  margin: 0.75rem 0 0;
  color: #57534e;
}

.home-field,
.home-button {
  border: 1px solid $line;
  border-radius: 0.5rem;
  padding: 0.65rem 0.85rem;
  font: inherit;
  color: inherit;
  background: #fff;
}

.home-button {
  border-color: $accent;
  color: #fff;
  cursor: pointer;
  background: $accent;
}

.home-message {
  min-height: 1.5rem;
  margin: 1rem 0 0;
}
</style>
