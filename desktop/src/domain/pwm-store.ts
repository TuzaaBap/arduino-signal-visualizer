import { isSequenceNewer } from "./gpio-store";
import type { PwmUpdate } from "./types";

export const PWM_HISTORY_LIMIT = 180;
export const UNO_PWM_PINS = [3, 5, 6, 9, 10, 11] as const;

export interface PwmPinState {
  latest: PwmUpdate;
  history: readonly PwmUpdate[];
}

export type PwmState = Readonly<Record<number, PwmPinState>>;

export function applyPwmUpdates(
  current: PwmState,
  updates: readonly PwmUpdate[],
  historyLimit = PWM_HISTORY_LIMIT,
): PwmState {
  if (updates.length === 0) {
    return current;
  }
  if (!Number.isInteger(historyLimit) || historyLimit < 1) {
    throw new Error("PWM history limit must be a positive integer");
  }

  const next: Record<number, PwmPinState> = { ...current };
  for (const update of updates) {
    if (!UNO_PWM_PINS.includes(update.pin as (typeof UNO_PWM_PINS)[number])) {
      continue;
    }
    const previous = next[update.pin];
    if (previous && !isSequenceNewer(previous.latest.sequence, update.sequence)) {
      continue;
    }
    const history = [...(previous?.history ?? []), update];
    next[update.pin] = {
      latest: update,
      history:
        history.length > historyLimit
          ? history.slice(history.length - historyLimit)
          : history,
    };
  }
  return next;
}

export function pwmFullScale(resolutionBits: number): number {
  if (resolutionBits !== 8) {
    throw new Error(`Unsupported Uno PWM resolution ${resolutionBits}`);
  }
  return 2 ** resolutionBits - 1;
}

export function pwmPercentage(update: PwmUpdate): number {
  return update.dutyPpm / 10_000;
}

export function pwmWaveformPath(update: PwmUpdate, windowNs: number): string {
  if (!Number.isFinite(windowNs) || windowNs <= 0) {
    throw new Error("PWM waveform window must be positive");
  }
  if (update.outputMode === "constantLow") {
    return "M 0 36 H 120";
  }
  if (update.outputMode === "constantHigh") {
    return "M 0 4 H 120";
  }
  if (
    update.periodNs === null ||
    update.highTimeNs === null ||
    update.lowTimeNs === null ||
    update.periodNs <= 0 ||
    update.highTimeNs <= 0 ||
    update.lowTimeNs <= 0
  ) {
    return "";
  }

  const xAt = (timeNs: number) => Math.min(120, (timeNs * 120) / windowNs);
  const commands = ["M 0 4"];
  let cursorNs = 0;
  while (cursorNs < windowNs) {
    cursorNs += update.highTimeNs;
    commands.push(`H ${xAt(cursorNs).toFixed(3)}`);
    if (cursorNs >= windowNs) {
      break;
    }
    commands.push("V 36");
    cursorNs += update.lowTimeNs;
    commands.push(`H ${xAt(cursorNs).toFixed(3)}`);
    if (cursorNs >= windowNs) {
      break;
    }
    commands.push("V 4");
  }
  return commands.join(" ");
}
