/**
 * Latest-only single-flight gate for scrubbing.
 * At most one `run` executes at a time; superseded schedules replace `pending`
 * so intermediate slider positions never start IPC/extract work.
 */
export type ScrubGate = {
  schedule: (key: string, run: () => Promise<void>) => void;
  /** 0 or 1 — how many `run` callbacks are executing. */
  inflight: () => number;
  /** Drop a queued (not yet started) scrub target. */
  clearPending: () => void;
};

type Work = { key: string; run: () => Promise<void> };

export function createScrubGate(): ScrubGate {
  let busy = false;
  let pending: Work | null = null;
  let inflightCount = 0;

  async function pump(initial: Work): Promise<void> {
    let current: Work | null = initial;
    busy = true;
    try {
      while (current !== null) {
        pending = null;
        inflightCount = 1;
        try {
          await current.run();
        } catch {
          // Callers surface errors inside `run`; keep draining the latest target.
        } finally {
          inflightCount = 0;
        }
        current = pending;
      }
    } finally {
      busy = false;
      inflightCount = 0;
    }
  }

  return {
    schedule(key, run) {
      const work: Work = { key, run };
      if (busy) {
        pending = work;
        return;
      }
      void pump(work);
    },
    inflight: () => inflightCount,
    clearPending() {
      pending = null;
    },
  };
}
