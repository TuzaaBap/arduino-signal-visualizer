import {
  adcFullScale,
  adcPercentage,
  adcVoltageMv,
  type AnalogChannelState,
} from "../domain/analog-store";

interface AnalogInspectorProps {
  channel: number;
  state: AnalogChannelState | undefined;
}

const referenceLabels = {
  default: "Default supply",
  internal: "Internal",
  external: "External",
} as const;

export function AnalogInspector({ channel, state }: AnalogInspectorProps) {
  const sample = state?.latest;
  const voltageMv = sample ? adcVoltageMv(sample) : null;

  return (
    <aside className="pin-inspector analog-inspector" aria-labelledby="analog-heading">
      <div className="pin-title-row">
        <div>
          <p className="eyebrow">Selected channel</p>
          <h2 id="analog-heading">A{channel}</h2>
        </div>
        <span className="logic-badge logic-badge--analog">
          {sample ? sample.rawValue : "NO DATA"}
        </span>
      </div>

      <dl className="pin-facts">
        <div>
          <dt>Raw ADC count</dt>
          <dd>
            {sample
              ? `${sample.rawValue} / ${adcFullScale(sample.resolutionBits)}`
              : "—"}
          </dd>
        </div>
        <div>
          <dt>Calculated voltage</dt>
          <dd>{voltageMv === null ? "—" : `${(voltageMv / 1_000).toFixed(4)} V`}</dd>
        </div>
        <div>
          <dt>Reference</dt>
          <dd>
            {sample
              ? `${referenceLabels[sample.referenceMode]}, ${
                  sample.referenceMv === 0
                    ? "unknown voltage"
                    : `${sample.referenceMv} mV`
                }`
              : "—"}
          </dd>
        </div>
        <div>
          <dt>Full scale</dt>
          <dd>{sample ? `${adcPercentage(sample).toFixed(2)}%` : "—"}</dd>
        </div>
        <div>
          <dt>Board time</dt>
          <dd>
            {sample
              ? `${(sample.boardTimestampUs / 1_000).toFixed(3)} ms`
              : "—"}
          </dd>
        </div>
        <div>
          <dt>Graph buffer</dt>
          <dd>{state?.history.length ?? 0} bounded samples</dd>
        </div>
      </dl>

      <p className="measurement-note">
        Voltage is calculated from raw counts and declared reference metadata.
        It is not a calibrated meter or oscilloscope measurement.
      </p>
    </aside>
  );
}
