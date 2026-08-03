import { describe, expect, it } from "vitest";
import { UNO_PWM_PINS } from "./pwm-store";
import {
  UNO_R3_ANALOG_PINS,
  UNO_R3_AUXILIARY_HEADER,
  UNO_R3_DIGITAL_PINS,
  describeCapabilities,
} from "./uno-r3-pinout";

describe("Uno R3 semantic pin map", () => {
  it("contains every physical digital and analog header pin exactly once", () => {
    expect(UNO_R3_DIGITAL_PINS).toHaveLength(14);
    expect(new Set(UNO_R3_DIGITAL_PINS.map(({ pin }) => pin)).size).toBe(14);
    expect(UNO_R3_DIGITAL_PINS.map(({ pin }) => pin)).toEqual([
      13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
    ]);

    expect(UNO_R3_ANALOG_PINS).toHaveLength(6);
    expect(UNO_R3_ANALOG_PINS.map(({ channel }) => channel)).toEqual([
      0, 1, 2, 3, 4, 5,
    ]);
  });

  it("matches the validated Uno hardware PWM set", () => {
    const pwmPins = UNO_R3_DIGITAL_PINS
      .filter(({ capabilities }) => capabilities.includes("pwm"))
      .map(({ pin }) => pin)
      .sort((a, b) => a - b);
    expect(pwmPins).toEqual([...UNO_PWM_PINS].sort((a, b) => a - b));
  });

  it("maps UART, external interrupts, SPI and I2C to the Uno pins", () => {
    const pinsWith = (capability: Parameters<typeof describeCapabilities>[0][number]) =>
      UNO_R3_DIGITAL_PINS
        .filter(({ capabilities }) => capabilities.includes(capability))
        .map(({ pin }) => pin)
        .sort((a, b) => a - b);

    expect(pinsWith("uart-rx")).toEqual([0]);
    expect(pinsWith("uart-tx")).toEqual([1]);
    expect(pinsWith("external-interrupt")).toEqual([2, 3]);
    expect(pinsWith("spi-ss")).toEqual([10]);
    expect(pinsWith("spi-mosi")).toEqual([11]);
    expect(pinsWith("spi-miso")).toEqual([12]);
    expect(pinsWith("spi-sck")).toEqual([13]);

    expect(
      UNO_R3_ANALOG_PINS.find(({ channel }) => channel === 4)?.capabilities,
    ).toContain("i2c-sda");
    expect(
      UNO_R3_ANALOG_PINS.find(({ channel }) => channel === 5)?.capabilities,
    ).toContain("i2c-scl");
    expect(UNO_R3_AUXILIARY_HEADER.map(({ aliasOf }) => aliasOf)).toEqual([
      "A5 / D19",
      "A4 / D18",
      undefined,
      undefined,
    ]);
  });
});
