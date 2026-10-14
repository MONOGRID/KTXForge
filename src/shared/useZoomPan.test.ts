import { describe, expect, it } from "vitest";
import { ref } from "vue";
import {
  useZoomPan,
  zoomPanTransformStyle,
  type ZoomMode,
} from "./useZoomPan";

/**
 * Module-level shared zoom state survives across useZoomPan() calls.
 * Reset via zoomFit between cases so tests do not leak mode.
 */
function api() {
  const width = ref(100);
  const height = ref(100);
  return useZoomPan(width, height, () => true);
}

describe("zoomPanTransformStyle", () => {
  it("centers pan=0 content in the frame at any Fit scale", () => {
    const frameW = 800;
    const frameH = 600;
    // Tiny mip, large Fit upscale.
    const nw = 5;
    const nh = 4;
    const scale = Math.min((frameW - 8) / nw, (frameH - 8) / nh);
    const style = zoomPanTransformStyle(0, 0, scale, nw, nh, frameW, frameH);
    expect(style.transformOrigin).toBe("0 0");
    const tx = (frameW - nw * scale) / 2;
    const ty = (frameH - nh * scale) / 2;
    expect(style.transform).toBe(
      `translate(${String(tx)}px, ${String(ty)}px) scale(${String(scale)})`,
    );
    expect(tx).toBeGreaterThan(0);
    expect(ty).toBe(4);
  });

  it("keeps 1×1 square centering (no %-translate shrink-to-fit trap)", () => {
    const frameW = 770;
    const frameH = 618;
    const scale = Math.min((frameW - 8) / 1, (frameH - 8) / 1);
    const style = zoomPanTransformStyle(0, 0, scale, 1, 1, frameW, frameH);
    const tx = (frameW - scale) / 2;
    const ty = (frameH - scale) / 2;
    expect(style.transform).toBe(
      `translate(${String(tx)}px, ${String(ty)}px) scale(${String(scale)})`,
    );
    // Height-limited Fit: equal side letterbox, pad-sized top/bottom.
    expect(tx).toBeGreaterThan(ty);
    expect(ty).toBe(4);
    expect(frameW - scale - 2 * tx).toBeCloseTo(0, 5);
  });

  it("adds pan on top of the centering offset", () => {
    const style = zoomPanTransformStyle(10, -20, 2, 100, 50, 400, 300);
    expect(style.transform).toBe("translate(110px, 80px) scale(2)");
  });
});

describe("useZoomPan Fit pan vs content size", () => {
  it("clears pan when content size changes while in Fit", () => {
    const width = ref(5);
    const height = ref(4);
    const zoom = useZoomPan(width, height, () => true);
    zoom.zoomFit();
    zoom.panX.value = 80;
    zoom.panY.value = 10;

    width.value = 2;
    height.value = 2;

    expect(zoom.zoomMode.value).toBe("fit");
    expect(zoom.panX.value).toBe(0);
    expect(zoom.panY.value).toBe(0);
  });
});

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
