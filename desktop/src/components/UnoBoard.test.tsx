import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { GpioUpdate } from "../domain/types";
import type { AnalogState } from "../domain/analog-store";
import { UnoBoard } from "./UnoBoard";

const d13High: GpioUpdate = {
  sequence: 4,
  boardTimestampUs: 500_000,
  pin: 13,
  direction: "output",
  level: "high",
  source: "write",
};

function renderBoard(
  tx: boolean,
  rx: boolean,
  d13Active: boolean,
  analog: AnalogState = {},
  activeTab: "digital" | "analog" | "pwm" = "digital",
): string {
  return renderToStaticMarkup(
    <UnoBoard
      pins={d13Active ? { 13: d13High } : {}}
      analog={analog}
      pwm={{}}
      serialLeds={{ tx, rx }}
      selectedDigitalPin={13}
      selectedAnalogChannel={0}
      selectedPwmPin={9}
      activeTab={activeTab}
      onSelectDigitalPin={() => undefined}
      onSelectAnalogChannel={() => undefined}
      onSelectPwmPin={() => undefined}
    />,
  );
}

describe("Uno board activity indicators", () => {
  it("renders TX, RX, and L from their independent observed states", () => {
    const markup = renderBoard(true, false, true);

    expect(markup).toContain('aria-label="TX serial activity active"');
    expect(markup).toContain('aria-label="RX serial activity inactive"');
    expect(markup).toContain('aria-label="L LED D13 active"');
  });

  it("renders every activity indicator inactive without observations", () => {
    const markup = renderBoard(false, false, false);

    expect(markup).toContain('aria-label="TX serial activity inactive"');
    expect(markup).toContain('aria-label="RX serial activity inactive"');
    expect(markup).toContain('aria-label="L LED D13 inactive"');
  });

  it("renders an active analog-input glow with the latest ADC level", () => {
    const sample = {
      sequence: 7,
      boardTimestampUs: 800_000,
      channel: 2,
      rawValue: 512,
      resolutionBits: 10,
      referenceMode: "default" as const,
      referenceMv: 5000,
    };
    const markup = renderBoard(
      false,
      false,
      false,
      { 2: { latest: sample, history: [sample] } },
      "analog",
    );

    expect(markup).toContain("board-pin--analog-active");
    expect(markup).toContain("analog-activity-ring");
    expect(markup).toContain("input active at 512 of 1023");
  });
});
