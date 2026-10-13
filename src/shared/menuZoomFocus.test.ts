import { describe, expect, it } from "vitest";
import { nextMenuZoomEligibility } from "./menuZoomFocus";

describe("nextMenuZoomEligibility", () => {
  it("becomes eligible when the window gains focus", () => {
    expect(nextMenuZoomEligibility(false, true, false)).toBe(true);
    expect(nextMenuZoomEligibility(false, true, true)).toBe(true);
  });

  it("keeps eligibility when blur is not caused by another app window", () => {
    expect(nextMenuZoomEligibility(true, false, false)).toBe(true);
  });

  it("drops eligibility when another app window is focused", () => {
    expect(nextMenuZoomEligibility(true, false, true)).toBe(false);
  });

  it("stays ineligible on blur when it was already ineligible", () => {
    expect(nextMenuZoomEligibility(false, false, false)).toBe(false);
  });
});
