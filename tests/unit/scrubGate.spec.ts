import { expect, test } from "@playwright/test";
import { createScrubGate } from "../../src/shared/scrubGate";

function sleep(ms: number): Promise<void> {
  return new Promise((r) => {
    setTimeout(r, ms);
  });
}

test.describe("createScrubGate", () => {
  test("coalesces a stampede to the first run plus the latest key", async () => {
    const gate = createScrubGate();
    let maxInflight = 0;
    let started = 0;
    const finished: string[] = [];

    const schedule = (key: string, delay: number): void => {
      gate.schedule(key, async () => {
        started += 1;
        maxInflight = Math.max(maxInflight, gate.inflight());
        await sleep(delay);
        finished.push(key);
      });
    };

    schedule("0", 40);
    for (let i = 1; i <= 19; i += 1) {
      schedule(String(i), 5);
    }

    await sleep(200);

    expect(maxInflight).toBeLessThanOrEqual(1);
    expect(started).toBeLessThanOrEqual(2);
    expect(finished[finished.length - 1]).toBe("19");
    expect(finished.filter((k) => k !== "0" && k !== "19")).toEqual([]);
  });
});
