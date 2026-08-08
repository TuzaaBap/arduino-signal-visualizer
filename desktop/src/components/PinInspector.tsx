import type { GpioUpdate } from "../domain/types";

interface PinInspectorProps {
  pin: number;
  state: GpioUpdate | undefined;
  nominalLogicMv: number;
}

const directionLabel: Record<GpioUpdate["direction"], string> = {
  input: "Input",
  output: "Output",
  inputPullup: "Input pull-up",
  unknown: "Not reported",
};

const sourceLabel: Record<GpioUpdate["source"], string> = {
  write: "digitalWrite (instrumented)",
  read: "digitalRead (instrumented)",
  modeChange: "pinMode (instrumented)",
};

export function PinInspector({
  pin,
  state,
  nominalLogicMv,
}: PinInspectorProps) {
  const estimatedMv = state?.level === "high" ? nominalLogicMv : 0;

  return (
    <aside className="pin-inspector" aria-labelledby="pin-heading">
      <div className="pin-title-row">
        <div>
          <p className="eyebrow">Selected pin</p>
          <h2 id="pin-heading">D{pin}</h2>
        </div>
        <span
          className={`logic-badge ${
            state?.level === "high" ? "logic-badge--high" : ""
          }`}
        >
          {state?.level?.toUpperCase() ?? "NO DATA"}
        </span>
      </div>

      <dl className="pin-facts">
        <div>
          <dt>Direction</dt>
          <dd>{state ? directionLabel[state.direction] : "Not reported"}</dd>
        </div>
        <div>
          <dt>Logic voltage</dt>
          <dd>{state ? `${(estimatedMv / 1_000).toFixed(2)} V` : "—"}</dd>
        </div>
        <div>
          <dt>Observed through</dt>
          <dd>{state ? sourceLabel[state.source] : "—"}</dd>
        </div>
        <div>
          <dt>Board time</dt>
          <dd>
            {state ? `${(state.boardTimestampUs / 1_000).toFixed(3)} ms` : "—"}
          </dd>
        </div>
        <div>
          <dt>Packet sequence</dt>
          <dd>{state?.sequence ?? "—"}</dd>
        </div>
      </dl>

      <p className="measurement-note">
        Voltage is a logic-level estimate, not a physical ADC measurement.
      </p>
      {(pin === 0 || pin === 1) && (
        <p className="serial-pin-note">
          D0/D1 carry the Uno UART. Using them can interfere with ASV USB serial.
        </p>
      )}
    </aside>
  );
}
