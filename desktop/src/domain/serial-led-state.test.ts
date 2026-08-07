import { describe, expect, it } from "vitest";

import {
  INACTIVE_SERIAL_LEDS,
  applySerialActivity,
  serialLedVisibility,
} from "./serial-led-state";

describe("serial LED state", () => {
  it("holds the Uno TX indicator for the backend-provided pulse duration", () => {
    const state = applySerialActivity(
      INACTIVE_SERIAL_LEDS,
      { txBytes: 12, rxBytes: 0, pulseDurationMs: 100 },
      1_000,
    );

    expect(serialLedVisibility(state, 1_099)).toEqual({ tx: true, rx: false });
    expect(serialLedVisibility(state, 1_100)).toEqual({ tx: false, rx: false });
  });

  it("extends an active pulse when more traffic arrives", () => {
    const first = applySerialActivity(
      INACTIVE_SERIAL_LEDS,
      { txBytes: 1, rxBytes: 0, pulseDurationMs: 100 },
      500,
    );
    const extended = applySerialActivity(
      first,
      { txBytes: 4, rxBytes: 0, pulseDurationMs: 100 },
      570,
    );

    expect(serialLedVisibility(extended, 650).tx).toBe(true);
    expect(serialLedVisibility(extended, 670).tx).toBe(false);
  });

  it("tracks RX and TX independently", () => {
    const state = applySerialActivity(
      INACTIVE_SERIAL_LEDS,
      { txBytes: 0, rxBytes: 3, pulseDurationMs: 100 },
      2_000,
    );

    expect(serialLedVisibility(state, 2_050)).toEqual({ tx: false, rx: true });
  });
});
