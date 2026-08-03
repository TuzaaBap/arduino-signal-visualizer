import {
  pwmFullScale,
  pwmPercentage,
  type PwmPinState,
} from "../domain/pwm-store";
import type { PwmOutputMode, PwmWaveformMode } from "../domain/types";
import { formatDuration, formatFrequency } from "./PwmPanel";

interface PwmInspectorProps {
  pin: number;
  state: PwmPinState | undefined;
}

const OUTPUT_MODE_LABELS: Record<PwmOutputMode, string> = {
  constantLow: "Constant LOW",
  hardwarePwm: "Hardware PWM",
  constantHigh: "Constant HIGH",
};

const WAVEFORM_MODE_LABELS: Record<PwmWaveformMode, string> = {
  fastPwm: "Fast PWM",
  phaseCorrectPwm: "Phase-correct PWM",
  phaseAndFrequencyCorrectPwm: "Phase/frequency-correct PWM",
};

export function PwmInspector({ pin, state }: PwmInspectorProps) {
  const update = state?.latest;

  return (
    <aside className="pin-inspector pwm-inspector" aria-labelledby="pwm-heading">
      <div className="pin-title-row">
        <div>
          <p className="eyebrow">Configured MCU waveform</p>
          <h2 id="pwm-heading">D{pin}</h2>
        </div>
        <span className="logic-badge logic-badge--pwm">
          {update ? `${pwmPercentage(update).toFixed(3)}%` : "NO DATA"}
        </span>
      </div>

      <dl className="pin-facts pwm-facts">
        <Fact label="analogWrite count" value={update ? `${update.dutyValue} / ${pwmFullScale(update.resolutionBits)}` : null} />
        <Fact label="Output mode" value={update ? OUTPUT_MODE_LABELS[update.outputMode] : null} />
        <Fact label="Frequency" value={update ? formatFrequency(update.frequencyMillihz) : null} />
        <Fact label="Period" value={update ? formatDuration(update.periodNs) : null} />
        <Fact label="HIGH time" value={update ? formatDuration(update.highTimeNs) : null} />
        <Fact label="LOW time" value={update ? formatDuration(update.lowTimeNs) : null} />
        <Fact label="Timer channel" value={update ? `Timer ${update.timerNumber}${update.timerChannel.toUpperCase()}` : null} />
        <Fact label="Waveform mode" value={update ? WAVEFORM_MODE_LABELS[update.waveformMode] : null} />
        <Fact label="Timer clock" value={update ? `${(update.timerClockHz / 1_000_000).toFixed(3)} MHz ÷ ${update.prescaler}` : null} />
        <Fact label="TOP / OCR / TCNT" value={update ? `${update.top} / ${update.compareValue} / ${update.counterValue}` : null} />
        <Fact label="TCCR A / B" value={update ? `${hexByte(update.controlA)} / ${hexByte(update.controlB)}` : null} />
        <Fact label="Board time" value={update ? `${(update.boardTimestampUs / 1_000).toFixed(3)} ms` : null} />
        <Fact label="Packet sequence" value={update ? String(update.sequence) : null} />
        <Fact label="History buffer" value={`${state?.history.length ?? 0} bounded states`} />
      </dl>

      <p className="measurement-note">
        This trace is calculated from the MCU timer configuration captured after
        ASV.analogWrite(). It represents the configured digital pulse train, not a
        voltage probe: loading, noise, rise/fall time, and oscillator tolerance
        still require measurement hardware.
      </p>
    </aside>
  );
}

function Fact({ label, value }: { label: string; value: string | null }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value ?? "—"}</dd>
    </div>
  );
}

function hexByte(value: number): string {
  return `0x${value.toString(16).padStart(2, "0").toUpperCase()}`;
}
