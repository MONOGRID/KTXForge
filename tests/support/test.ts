import process from "node:process";
import { test as base } from "@playwright/test";
import { Ipc } from "./ipc";

/**
 * `driver`: the `ktxforge-e2e` binary global setup built.
 * `ipc`: one headless app per worker, shared by its tests like one app session.
 */
export const test = base.extend<object, { driver: string; ipc: Ipc }>({
  driver: [process.env.KTXFORGE_E2E_DRIVER ?? "", { scope: "worker", option: true }],
  ipc: [
    async ({ driver }, use) => {
      const ipc = new Ipc(driver);
      await use(ipc);
      await ipc.close();
    },
    { scope: "worker" },
  ],
});

export { expect } from "@playwright/test";
