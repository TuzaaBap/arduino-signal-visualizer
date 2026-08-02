import { describe, expect, it } from "vitest";

import {
  adcFullScale,
  adcPercentage,
  adcVoltageMv,
  applyAdcSamples,
} from "./analog-store";
import type { AdcSample } from "./types";

const sample = (
  channel: number,
  sequence: number,
  rawValue: number,
): AdcSample => ({
  channel,
  sequence,
  rawValue,
  boardTimestampUs: sequence * 1_000,
  resolutionBits: 10,
  referenceMode: "default",
  referenceMv: 5_000,
});

describe("analog store", () => {
  it("keeps every new sample up to the per-channel bound", () => {
    const state = applyAdcSamples(
      {},
      [sample(0, 1, 100), sample(0, 2, 200), sample(0, 3, 300)],
      2,
    );
    expect(state[0]?.latest.rawValue).toBe(300);
    expect(state[0]?.history.map((entry) => entry.rawValue)).toEqual([200, 300]);
  });

  it("ignores invalid channels and stale samples", () => {
    const current = applyAdcSamples({}, [sample(0, 10, 500)]);
    const next = applyAdcSamples(current, [
      sample(0, 9, 400),
      sample(6, 11, 600),
    ]);
    expect(next).toEqual(current);
  });

  it("calculates full scale, voltage, and percentage from metadata", () => {
    const midpoint = sample(0, 1, 512);
    expect(adcFullScale(10)).toBe(1_023);
    expect(adcVoltageMv(midpoint)).toBeCloseTo(2_502.44, 2);
    expect(adcPercentage(midpoint)).toBeCloseTo(50.049, 3);
    expect(adcVoltageMv({ ...midpoint, referenceMv: 0 })).toBeNull();
  });
});
