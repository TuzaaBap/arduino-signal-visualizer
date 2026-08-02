import { isSequenceNewer } from "./gpio-store";
import type { AdcSample } from "./types";

export const ANALOG_HISTORY_LIMIT = 180;

export interface AnalogChannelState {
  latest: AdcSample;
  history: readonly AdcSample[];
}

export type AnalogState = Readonly<Record<number, AnalogChannelState>>;

export function applyAdcSamples(
  current: AnalogState,
  samples: readonly AdcSample[],
  historyLimit = ANALOG_HISTORY_LIMIT,
): AnalogState {
  if (samples.length === 0) {
    return current;
  }
  if (!Number.isInteger(historyLimit) || historyLimit < 1) {
    throw new Error("ADC history limit must be a positive integer");
  }

  const next: Record<number, AnalogChannelState> = { ...current };
  for (const sample of samples) {
    if (sample.channel < 0 || sample.channel > 5) {
      continue;
    }
    const previous = next[sample.channel];
    if (
      previous &&
      !isSequenceNewer(previous.latest.sequence, sample.sequence)
    ) {
      continue;
    }
    const history = [...(previous?.history ?? []), sample];
    next[sample.channel] = {
      latest: sample,
      history:
        history.length > historyLimit
          ? history.slice(history.length - historyLimit)
          : history,
    };
  }
  return next;
}

export function adcFullScale(resolutionBits: number): number {
  if (![8, 10, 12, 14, 16].includes(resolutionBits)) {
    throw new Error(`Unsupported ADC resolution ${resolutionBits}`);
  }
  return 2 ** resolutionBits - 1;
}

export function adcVoltageMv(sample: AdcSample): number | null {
  if (sample.referenceMv === 0) {
    return null;
  }
  return (sample.rawValue * sample.referenceMv) / adcFullScale(sample.resolutionBits);
}

export function adcPercentage(sample: AdcSample): number {
  return (sample.rawValue * 100) / adcFullScale(sample.resolutionBits);
}
