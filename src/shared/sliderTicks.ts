const NICE_INTERVALS = [1, 2, 5, 10, 20, 25, 50, 100] as const;

function denseTicks(min: number, max: number): number[] {
  return Array.from({ length: max - min + 1 }, (_, i) => min + i);
}

function niceInterval(span: number, maxTicks: number): number {
  const rough = Math.ceil(span / (maxTicks - 1));
  return NICE_INTERVALS.find((candidate) => candidate >= rough) ?? rough;
}

function sparseTicks(min: number, max: number, interval: number): number[] {
  const ticks: number[] = [];
  for (let value = min; value < max; value += interval) {
    ticks.push(value);
  }
  if (ticks[ticks.length - 1] !== max) {
    ticks.push(max);
  }
  return ticks;
}

/** Integer tick values from `min` to `max`, thinned to about `maxTicks` labels. */
export function rangeTickValues(
  min: number,
  max: number,
  maxTicks = 8,
): number[] {
  if (max <= min) {
    return [min];
  }

  const span = max - min;
  if (span + 1 <= maxTicks) {
    return denseTicks(min, max);
  }

  return sparseTicks(min, max, niceInterval(span, maxTicks));
}
