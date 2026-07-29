import { describe, expect, it } from "vitest";

import { applyGpioUpdates, isSequenceNewer } from "./gpio-store";
import type { GpioUpdate } from "./types";

const update = (pin: number, sequence: number): GpioUpdate => ({
  pin,
  sequence,
  boardTimestampUs: sequence * 10,
  direction: "output",
  level: "high",
  source: "write",
});

describe("GPIO store", () => {
  it("applies the latest update for each digital pin", () => {
    const state = applyGpioUpdates({}, [update(2, 1), update(13, 2)]);
    expect(state[2]?.sequence).toBe(1);
    expect(state[13]?.sequence).toBe(2);
  });

  it("ignores stale and out-of-range updates", () => {
    const current = { 4: update(4, 10) };
    const state = applyGpioUpdates(current, [update(4, 9), update(14, 11)]);
    expect(state).toEqual(current);
  });

  it("accepts sequence wrap-around", () => {
    expect(isSequenceNewer(0xffff, 0)).toBe(true);
    expect(isSequenceNewer(10, 10)).toBe(false);
    expect(isSequenceNewer(10, 9)).toBe(false);
  });
});

