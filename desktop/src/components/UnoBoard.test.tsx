import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import type { GpioUpdate } from "../domain/types";
import { UnoBoard } from "./UnoBoard";

const d13High: GpioUpdate = {
  sequence: 4,
  boardTimestampUs: 500_000,
  pin: 13,
  direction: "output",
  level: "high",
  source: "write",
};

function renderBoard(tx: boolean, rx: boolean, d13Active: boolean): string {
  return renderToStaticMarkup(
    <UnoBoard
      pins={d13Active ? { 13: d13High } : {}}
      pwm={{}}
      serialLeds={{ tx, rx }}
      selectedDigitalPin={13}
      selectedAnalogChannel={0}
      selectedPwmPin={9}
      activeTab="digital"
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
});
