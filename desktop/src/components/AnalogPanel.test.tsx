import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { AnalogPanel } from "./AnalogPanel";

describe("AnalogPanel", () => {
  it("labels sampled channels as active and states the 10 Hz display limit", () => {
    const sample = {
      sequence: 3,
      boardTimestampUs: 20_000,
      channel: 0,
      rawValue: 205,
      resolutionBits: 10,
      referenceMode: "default" as const,
      referenceMv: 5000,
    };
    const markup = renderToStaticMarkup(
      <AnalogPanel
        channels={{ 0: { latest: sample, history: [sample] } }}
        selectedChannel={0}
        mockMode={false}
        onSelectChannel={() => undefined}
      />,
    );

    expect(markup).toContain("Input active");
    expect(markup).toContain("Waveform display limit:");
    expect(markup).toContain("validated for recognizable waveform shape up to 10 Hz");
  });
});
