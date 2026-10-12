import { describe, expect, it } from "vitest";
import { ref } from "vue";
import { useZoomPan, type ZoomMode } from "./useZoomPan";

/**
 * Module-level shared zoom state survives across useZoomPan() calls.
 * Reset via zoomFit between cases so tests do not leak mode.
 */
function api() {
  const width = ref(100);
  const height = ref(100);
  return useZoomPan(width, height, () => true);
}

describe("useZoomPan sticky modes", () => {
  it("Fit and Actual are sticky and mutually exclusive; wheel clears both", () => {
    const zoom = api();
    zoom.zoomFit();
    expect(zoom.zoomMode.value).toBe("fit" satisfies ZoomMode);

    zoom.zoomActual();
    expect(zoom.zoomMode.value).toBe("actual");

    zoom.zoomFit();
    expect(zoom.zoomMode.value).toBe("fit");

    zoom.zoomIn();
    expect(zoom.zoomMode.value).toBe("manual");

    zoom.zoomActual();
    expect(zoom.zoomMode.value).toBe("actual");

    zoom.zoomOut();
    expect(zoom.zoomMode.value).toBe("manual");
  });

  it("Actual is 1:1 and Fit follows frame; manual keeps a free scale", () => {
    const width = ref(200);
    const height = ref(100);
    const zoom = useZoomPan(width, height, () => true);

    // Drive element size via the frame ref's mock — useZoomPan uses
    // useElementSize(frameRef). Without a DOM node, fitScale stays 1.
    zoom.zoomActual();
    expect(zoom.scale.value).toBe(1);

    zoom.zoomFit();
    expect(zoom.zoomMode.value).toBe("fit");

    zoom.zoomIn();
    const manual = zoom.scale.value;
    expect(zoom.zoomMode.value).toBe("manual");
    expect(manual).toBeGreaterThan(0);

    width.value = 50;
    height.value = 25;
    expect(zoom.zoomMode.value).toBe("manual");
    expect(zoom.scale.value).toBe(manual);
  });
});
