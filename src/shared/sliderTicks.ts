const NICE_INTERVALS = [1, 2, 5, 10, 20, 25, 50, 100] as const;

/** Integer tick values from `min` to `max`, thinned to about `maxTicks` labels. */
export function rangeTickValues(
  min: number,
  max: number,
  maxTicks = 8,
): number[] {
  if (max < min) {
    return [min];
  }
  if (max === min) {
    return [min];
  }

  const span = max - min;
  if (span + 1 <= maxTicks) {
    return Array.from({ length: span + 1 }, (_, i) => min + i);
  }

  const rough = Math.ceil(span / (maxTicks - 1));
  const interval =
    NICE_INTERVALS.find((candidate) => candidate >= rough) ?? rough;

  const ticks: number[] = [];
  for (let value = min; value < max; value += interval) {
    ticks.push(value);
  }
  if (ticks[ticks.length - 1] !== max) {
    ticks.push(max);
  }
  return ticks;
}
