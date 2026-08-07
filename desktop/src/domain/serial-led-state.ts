import type { SerialActivityBatch } from "./types";

export interface SerialLedDeadlines {
  txActiveUntilMs: number;
  rxActiveUntilMs: number;
}

export interface SerialLedVisibility {
  tx: boolean;
  rx: boolean;
}

export const INACTIVE_SERIAL_LEDS: SerialLedDeadlines = {
  txActiveUntilMs: 0,
  rxActiveUntilMs: 0,
};

const MAX_ACCEPTED_PULSE_MS = 1_000;

export function applySerialActivity(
  current: SerialLedDeadlines,
  activity: SerialActivityBatch,
  observedAtMs: number,
): SerialLedDeadlines {
  const pulseDurationMs = Number.isFinite(activity.pulseDurationMs)
    ? Math.min(MAX_ACCEPTED_PULSE_MS, Math.max(1, activity.pulseDurationMs))
    : 1;
  const activeUntilMs = observedAtMs + pulseDurationMs;

  return {
    txActiveUntilMs:
      activity.txBytes > 0
        ? Math.max(current.txActiveUntilMs, activeUntilMs)
        : current.txActiveUntilMs,
    rxActiveUntilMs:
      activity.rxBytes > 0
        ? Math.max(current.rxActiveUntilMs, activeUntilMs)
        : current.rxActiveUntilMs,
  };
}

export function serialLedVisibility(
  deadlines: SerialLedDeadlines,
  nowMs: number,
): SerialLedVisibility {
  return {
    tx: deadlines.txActiveUntilMs > nowMs,
    rx: deadlines.rxActiveUntilMs > nowMs,
  };
}
