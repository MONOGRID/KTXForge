import { availableParallelism } from "node:os";
import { join } from "node:path";
import process from "node:process";
import { defineConfig, devices } from "@playwright/test";
import { repoRoot, resultsDir } from "./support/paths";

/**
 * Test suite (`tests/unit` for frontend logic, `tests/e2e` for the rest): real inputs → the app's Rust pipeline + the bundled `ktx` →
 * real output files, checked with Khronos' own `ktx`; plus the real frontend in
 * Chromium with its IPC bridged to the same Rust (`gui.*.spec.ts`).
 */
export default defineConfig({
  testDir: ".",
  outputDir: join(resultsDir, "artifacts"),
  globalSetup: "./global-setup.ts",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  // Each test runs multi-threaded encodes; keep the box responsive.
  workers: Math.max(1, Math.floor(availableParallelism() / 4)),
  timeout: 120_000,
  reporter: [
    [process.env.CI ? "github" : "list"],
    ["html", { open: "never", outputFolder: join(resultsDir, "report") }],
  ],
  // Snapshots are created on first run and compared afterwards.
  updateSnapshots: "missing",
  snapshotPathTemplate: "{testDir}/__snapshots__/{testFilePath}/{arg}{ext}",
  expect: {
    toMatchSnapshot: { maxDiffPixelRatio: 0.01 },
  },
  projects: [
    // Pure frontend logic (src/); no app, no browser.
    { name: "unit", testMatch: /unit\/.*\.spec\.ts/ },
    // Inputs → Rust → files; no browser.
    { name: "pipeline", testMatch: /e2e\/(?!gui\.).*\.spec\.ts/ },
    // The real frontend in Chromium, IPC bridged to the same Rust.
    {
      name: "gui",
      testMatch: /e2e\/gui\..*\.spec\.ts/,
      use: { ...devices["Desktop Chrome"], baseURL: "http://127.0.0.1:1420" },
    },
  ],
  webServer: {
    command: "npx vite --host 127.0.0.1 --port 1420 --strictPort",
    cwd: repoRoot,
    url: "http://127.0.0.1:1420/inspector.html",
    reuseExistingServer: !process.env.CI,
  },
});
