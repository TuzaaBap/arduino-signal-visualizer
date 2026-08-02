import {
  adcFullScale,
  adcPercentage,
  adcVoltageMv,
  type AnalogState,
} from "../domain/analog-store";
import type { AdcSample } from "../domain/types";

interface AnalogPanelProps {
  channels: AnalogState;
  selectedChannel: number;
  mockMode: boolean;
  onSelectChannel: (channel: number) => void;
}

const CHANNELS = [0, 1, 2, 3, 4, 5] as const;

export function AnalogPanel({
  channels,
  selectedChannel,
  mockMode,
  onSelectChannel,
}: AnalogPanelProps) {
  return (
    <section className="analog-panel" aria-labelledby="analog-panel-heading">
      <div className="analog-panel-heading">
        <div>
          <p className="eyebrow">ADC channels</p>
          <h3 id="analog-panel-heading">Live analog values</h3>
        </div>
        {mockMode && <span className="mock-badge">MOCK DATA</span>}
      </div>

      <div className="analog-grid">
        {CHANNELS.map((channel) => {
          const state = channels[channel];
          return (
            <button
              className={`analog-card ${
                selectedChannel === channel ? "analog-card--selected" : ""
              }`}
              type="button"
              key={channel}
              onClick={() => onSelectChannel(channel)}
              aria-pressed={selectedChannel === channel}
            >
              <div className="analog-card-title">
                <strong>A{channel}</strong>
                <span>{state ? `${state.latest.resolutionBits}-bit` : "No data"}</span>
              </div>
              <AnalogValues sample={state?.latest} />
              <Sparkline
                samples={state?.history ?? []}
                resolutionBits={state?.latest.resolutionBits ?? 10}
              />
            </button>
          );
        })}
      </div>

      <p className="analog-disclaimer">
        Trend graphs show instrumented ADC samples, not oscilloscope-grade
        timing, bandwidth, or calibrated accuracy.
      </p>
    </section>
  );
}

function AnalogValues({ sample }: { sample: AdcSample | undefined }) {
  if (!sample) {
    return (
      <div className="analog-values analog-values--empty">
        Waiting for ASV.analogRead()
      </div>
    );
  }
  const voltageMv = adcVoltageMv(sample);
  return (
    <dl className="analog-values">
      <div>
        <dt>Raw</dt>
        <dd>
          {sample.rawValue} / {adcFullScale(sample.resolutionBits)}
        </dd>
      </div>
      <div>
        <dt>Voltage</dt>
        <dd>{voltageMv === null ? "Unknown" : `${(voltageMv / 1_000).toFixed(3)} V`}</dd>
      </div>
      <div>
        <dt>Reference</dt>
        <dd>{sample.referenceMv === 0 ? "External / unknown" : `${sample.referenceMv} mV`}</dd>
      </div>
      <div>
        <dt>Full scale</dt>
        <dd>{adcPercentage(sample).toFixed(1)}%</dd>
      </div>
    </dl>
  );
}

function Sparkline({
  samples,
  resolutionBits,
}: {
  samples: readonly AdcSample[];
  resolutionBits: number;
}) {
  const maximum = adcFullScale(resolutionBits);
  const points = samples
    .map((sample, index) => {
      const x = samples.length <= 1 ? 0 : (index * 120) / (samples.length - 1);
      const y = 38 - (sample.rawValue * 36) / maximum;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  return (
    <svg
      className="analog-sparkline"
      viewBox="0 0 120 40"
      role="img"
      aria-label={`${samples.length} recent ADC samples`}
      preserveAspectRatio="none"
    >
      <path d="M0 38 H120 M0 20 H120 M0 2 H120" />
      {points && <polyline points={points} />}
    </svg>
  );
}
