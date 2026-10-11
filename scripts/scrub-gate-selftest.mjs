/**
 * Red/green loop for scrub coalescing (no vitest in this repo).
 * Run: node scripts/scrub-gate-selftest.mjs
 */
import { createScrubGate } from "../src/shared/scrubGate.ts";

function assert(cond, msg) {
  if (!cond) {
    throw new Error(msg);
  }
}

async function sleep(ms) {
  await new Promise((r) => setTimeout(r, ms));
}

async function main() {
  const gate = createScrubGate();
  let maxInflight = 0;
  let started = 0;
  const finished = [];

  const schedule = (key, delay) => {
    gate.schedule(key, async () => {
      started += 1;
      maxInflight = Math.max(maxInflight, gate.inflight());
      await sleep(delay);
      finished.push(key);
    });
  };

  // Stampede: 20 rapid scrubs while first is slow.
  schedule("0", 40);
  for (let i = 1; i <= 19; i += 1) {
    schedule(String(i), 5);
  }

  await sleep(200);

  assert(maxInflight <= 1, `expected max inflight 1, got ${maxInflight}`);
  assert(
    started <= 2,
    `expected at most 2 runs (first + latest), got ${started}`,
  );
  assert(
    finished[finished.length - 1] === "19",
    `expected last finished key 19, got ${JSON.stringify(finished)}`,
  );
  assert(
    !finished.includes("1") || finished[0] === "0",
    "intermediate keys must not all run",
  );
  // Keys 1..18 must not have started (only 0 and 19).
  const extras = finished.filter((k) => k !== "0" && k !== "19");
  assert(extras.length === 0, `unexpected finished keys: ${extras.join(",")}`);

  console.log("scrub-gate-selftest: PASS");
}

main().catch((err) => {
  console.error("scrub-gate-selftest: FAIL", err);
  process.exit(1);
});
