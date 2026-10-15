import { expect, test } from "@playwright/test";
import { nextMenuZoomEligibility } from "../../src/shared/menuZoomFocus";

test.describe("nextMenuZoomEligibility", () => {
  test("becomes eligible when the window gains focus", () => {
    expect(nextMenuZoomEligibility(false, true, false)).toBe(true);
    expect(nextMenuZoomEligibility(false, true, true)).toBe(true);
  });

  test("keeps eligibility when blur is not caused by another app window", () => {
    expect(nextMenuZoomEligibility(true, false, false)).toBe(true);
  });

  test("drops eligibility when another app window is focused", () => {
    expect(nextMenuZoomEligibility(true, false, true)).toBe(false);
  });

  test("stays ineligible on blur when it was already ineligible", () => {
    expect(nextMenuZoomEligibility(false, false, false)).toBe(false);
  });
});
