import { useState } from "react";

import {
  pwmFullScale,
  pwmPercentage,
  pwmWaveformPath,
  UNO_PWM_PINS,
  type PwmState,
} from "../domain/pwm-store";
import type {
  PwmOutputMode,
  PwmUpdate,
  PwmWaveformMode,
} from "../domain/types";

interface PwmPanelProps {
  pins: PwmState;
  selectedPin: number;
  mockMode: boolean;
  onSelectPin: (pin: number) => void;
}

const TIMEBASE_WINDOWS_NS = [1_000_000, 2_500_000, 5_000_000, 10_000_000];

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

export function PwmPanel({
  pins,
  selectedPin,
  mockMode,
  onSelectPin,
}: PwmPanelProps) {
  const [timebaseIndex, setTimebaseIndex] = useState(2);
  const windowNs = TIMEBASE_WINDOWS_NS[timebaseIndex] ?? 5_000_000;

  return (
    <section className="analog-panel pwm-panel" aria-labelledby="pwm-panel-heading">
      <div className="analog-panel-heading pwm-panel-heading">
        <div>
          <p className="eyebrow">Hardware PWM pins</p>
          <h3 id="pwm-panel-heading">Configured MCU waveform</h3>
        </div>
        <div className="pwm-panel-actions">
          <span className="configured-badge">TIMER-DERIVED</span>
          {mockMode && <span className="mock-badge">MOCK DATA</span>}
          <div className="timebase-control" aria-label="PWM waveform time window">
            <button
              type="button"
              onClick={() => setTimebaseIndex((value) => Math.max(0, value - 1))}
              disabled={timebaseIndex === 0}
              aria-label="Zoom in PWM waveform"
            >
              −
            </button>
            <span>{formatDuration(windowNs)} window</span>
            <button
              type="button"
              onClick={() =>
                setTimebaseIndex((value) =>
                  Math.min(TIMEBASE_WINDOWS_NS.length - 1, value + 1),
                )
              }
              disabled={timebaseIndex === TIMEBASE_WINDOWS_NS.length - 1}
              aria-label="Zoom out PWM waveform"
            >
              +
            </button>
          </div>
        </div>
      </div>

      <div className="analog-grid pwm-grid">
        {UNO_PWM_PINS.map((pin) => {
          const state = pins[pin];
          return (
            <button
              className={`analog-card pwm-card ${
                selectedPin === pin ? "analog-card--selected" : ""
              }`}
              type="button"
              key={pin}
              onClick={() => onSelectPin(pin)}
              aria-pressed={selectedPin === pin}
            >
              <div className="analog-card-title">
                <strong>D{pin}</strong>
                <span>{state ? OUTPUT_MODE_LABELS[state.latest.outputMode] : "No data"}</span>
              </div>
              <PwmValues update={state?.latest} />
              <PwmTrace update={state?.latest} windowNs={windowNs} />
            </button>
          );
        })}
      </div>

      <p className="analog-disclaimer">
        Square waves are reconstructed from the ATmega328P timer registers captured
        by ASV. A logic analyzer should show the same configured timing within board
        clock and measurement tolerances; electrical voltage, noise, and edge shape
        are not measured.
      </p>
    </section>
  );
}

function PwmValues({ update }: { update: PwmUpdate | undefined }) {
  if (!update) {
    return (
      <div className="analog-values analog-values--empty">
        Waiting for ASV.analogWrite()
      </div>
    );
  }
  return (
    <dl className="analog-values">
      <div>
        <dt>Configured duty</dt>
        <dd>{pwmPercentage(update).toFixed(3)}%</dd>
      </div>
      <div>
        <dt>Frequency</dt>
        <dd>{formatFrequency(update.frequencyMillihz)}</dd>
      </div>
      <div>
        <dt>Period</dt>
        <dd>{formatDuration(update.periodNs)}</dd>
      </div>
      <div>
        <dt>Timer</dt>
        <dd>
          T{update.timerNumber}{update.timerChannel.toUpperCase()} · {WAVEFORM_MODE_LABELS[update.waveformMode]}
        </dd>
      </div>
      <div className="pwm-requested-count">
        <dt>analogWrite</dt>
        <dd>
          {update.dutyValue} / {pwmFullScale(update.resolutionBits)}
        </dd>
      </div>
    </dl>
  );
}

function PwmTrace({
  update,
  windowNs,
}: {
  update: PwmUpdate | undefined;
  windowNs: number;
}) {
  const trace = update ? pwmWaveformPath(update, windowNs) : "";
  return (
    <svg
      className="pwm-waveform"
      viewBox="0 0 120 40"
      role="img"
      aria-label={
        update
          ? `Configured D${update.pin} ${OUTPUT_MODE_LABELS[update.outputMode]} waveform over ${formatDuration(windowNs)}`
          : "No configured PWM waveform received"
      }
      preserveAspectRatio="none"
    >
      <path className="waveform-grid" d="M0 4 H120 M0 20 H120 M0 36 H120" />
      {trace && <path className="waveform-trace" d={trace} />}
    </svg>
  );
}

export function formatFrequency(frequencyMillihz: number | null): string {
  return frequencyMillihz === null
    ? "No carrier"
    : `${(frequencyMillihz / 1_000).toFixed(3)} Hz`;
}

export function formatDuration(durationNs: number | null): string {
  if (durationNs === null) {
    return "Not periodic";
  }
  if (durationNs < 1_000) {
    return `${durationNs.toFixed(0)} ns`;
  }
  if (durationNs < 1_000_000) {
    return `${(durationNs / 1_000).toFixed(3)} µs`;
  }
  return `${(durationNs / 1_000_000).toFixed(3)} ms`;
}
