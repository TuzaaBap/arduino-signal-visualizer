import { describe, expect, it } from "vitest";

import {
  applyPwmUpdates,
  pwmPercentage,
  pwmWaveformPath,
  type PwmState,
} from "./pwm-store";
import type { PwmUpdate } from "./types";

function update(sequence: number, pin = 9, dutyValue = 128): PwmUpdate {
  return {
    sequence,
    boardTimestampUs: sequence * 1_000,
    pin,
    dutyValue,
    resolutionBits: 8,
    outputMode:
      dutyValue === 0
        ? "constantLow"
        : dutyValue === 255
          ? "constantHigh"
          : "hardwarePwm",
    timerNumber: 1,
    timerChannel: "a",
    waveformMode: "phaseCorrectPwm",
    outputPolarity:
      dutyValue === 0 || dutyValue === 255 ? "disconnected" : "nonInverting",
    timerClockHz: 16_000_000,
    prescaler: 64,
    top: 255,
    compareValue: dutyValue,
    counterValue: 42,
    controlA: dutyValue === 0 || dutyValue === 255 ? 0x01 : 0x81,
    controlB: 0x03,
    periodNs: dutyValue === 0 || dutyValue === 255 ? null : 2_040_000,
    highTimeNs:
      dutyValue === 0 || dutyValue === 255 ? null : dutyValue * 8_000,
    lowTimeNs:
      dutyValue === 0 || dutyValue === 255
        ? null
        : 2_040_000 - dutyValue * 8_000,
    frequencyMillihz:
      dutyValue === 0 || dutyValue === 255 ? null : 490_196,
    dutyPpm:
      dutyValue === 255
        ? 1_000_000
        : dutyValue === 0
          ? 0
          : Math.round((dutyValue * 1_000_000) / 255),
  };
}

describe("PWM store", () => {
  it("retains the newest update and a bounded history", () => {
    let state: PwmState = {};
    state = applyPwmUpdates(state, [update(1), update(2, 9, 191)], 2);
    state = applyPwmUpdates(state, [update(3, 9, 255)], 2);

    expect(state[9]?.latest.dutyValue).toBe(255);
    expect(state[9]?.history.map((item) => item.sequence)).toEqual([2, 3]);
  });

  it("ignores stale updates and non-PWM Uno pins", () => {
    let state = applyPwmUpdates({}, [update(10)]);
    state = applyPwmUpdates(state, [update(9, 9, 64), update(11, 8, 64)]);

    expect(state[9]?.latest.sequence).toBe(10);
    expect(state[8]).toBeUndefined();
  });

  it("calculates percentage from integer duty metadata", () => {
    expect(pwmPercentage(update(1, 9, 0))).toBe(0);
    expect(pwmPercentage(update(2, 9, 255))).toBe(100);
    expect(pwmPercentage(update(3))).toBeCloseTo(50.196, 3);

    const fastPwm: PwmUpdate = {
      ...update(4, 5, 128),
      timerNumber: 0,
      timerChannel: "b",
      waveformMode: "fastPwm",
      periodNs: 1_024_000,
      highTimeNs: 512_000,
      lowTimeNs: 512_000,
      frequencyMillihz: 976_563,
      dutyPpm: 500_000,
    };
    expect(pwmPercentage(fastPwm)).toBe(50);
  });

  it("renders a rectangular timer waveform without diagonal interpolation", () => {
    const path = pwmWaveformPath(update(3), 5_000_000);

    expect(path).toContain("H");
    expect(path).toContain("V 36");
    expect(path).toContain("V 4");
    expect(path).not.toContain("L");
  });

  it("renders endpoint states as constant logic levels", () => {
    expect(pwmWaveformPath(update(1, 9, 0), 5_000_000)).toBe("M 0 36 H 120");
    expect(pwmWaveformPath(update(2, 9, 255), 5_000_000)).toBe("M 0 4 H 120");
  });
});
