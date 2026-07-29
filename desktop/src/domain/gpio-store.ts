import type { GpioUpdate } from "./types";

export type GpioState = Readonly<Record<number, GpioUpdate>>;

export function applyGpioUpdates(
  current: GpioState,
  updates: readonly GpioUpdate[],
): GpioState {
  if (updates.length === 0) {
    return current;
  }

  const next: Record<number, GpioUpdate> = { ...current };
  for (const update of updates) {
    if (update.pin < 0 || update.pin > 13) {
      continue;
    }
    const previous = next[update.pin];
    if (previous && !isSequenceNewer(previous.sequence, update.sequence)) {
      continue;
    }
    next[update.pin] = update;
  }
  return next;
}

export function isSequenceNewer(previous: number, candidate: number): boolean {
  const difference = (candidate - previous + 0x1_0000) & 0xffff;
  return difference > 0 && difference < 0x8000;
}

