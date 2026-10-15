import vue from "@vitejs/plugin-vue";
import { resolve } from "node:path";
import process from "node:process";
import UnoCSS from "unocss/vite";
import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;
const srcRoot = resolve(import.meta.dirname, "src");

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [vue(), UnoCSS()],
  resolve: {
    alias: {
      "@": srcRoot,
    },
  },
  clearScreen: false,
  build: {
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        inspector: resolve(import.meta.dirname, "inspector.html"),
      },
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
}));
