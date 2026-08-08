export type ConnectionPhase =
  | "disconnected"
  | "waitingForHello"
  | "connected"
  | "error";

export type ConnectionMode = "serial" | "mock";

export interface ConnectionStatus {
  phase: ConnectionPhase;
  mode: ConnectionMode | null;
  portName: string | null;
  detail: string;
}

export interface SerialPortDescriptor {
  name: string;
  kind: string;
  usbVendorId: number | null;
  usbProductId: number | null;
  manufacturer: string | null;
  product: string | null;
  serialNumber: string | null;
}

export type GpioDirection = "input" | "output" | "inputPullup" | "unknown";
export type GpioLevel = "low" | "high";
export type GpioObservationSource = "write" | "read" | "modeChange";

export interface GpioUpdate {
  sequence: number;
  boardTimestampUs: number;
  pin: number;
  direction: GpioDirection;
  level: GpioLevel;
  source: GpioObservationSource;
}

export interface GpioBatch {
  updates: GpioUpdate[];
  droppedUiEvents: number;
}

export interface SerialActivityBatch {
  /** Bytes transmitted by the Uno USB bridge and received by the desktop. */
  txBytes: number;
  /** Bytes received by the Uno USB bridge from the desktop. */
  rxBytes: number;
  pulseDurationMs: number;
}

export interface UserSerialBatch {
  bytes: number[];
  droppedBytes: number;
}

export type AdcReferenceMode = "default" | "internal" | "external";

export interface AdcSample {
  sequence: number;
  boardTimestampUs: number;
  channel: number;
  rawValue: number;
  resolutionBits: number;
  referenceMode: AdcReferenceMode;
  referenceMv: number;
}

export interface AdcBatch {
  samples: AdcSample[];
  coalescedUiSamples: number;
}

export type PwmOutputMode = "constantLow" | "hardwarePwm" | "constantHigh";
export type PwmTimerChannel = "a" | "b";
export type PwmWaveformMode =
  | "fastPwm"
  | "phaseCorrectPwm"
  | "phaseAndFrequencyCorrectPwm";
export type PwmOutputPolarity =
  | "disconnected"
  | "nonInverting"
  | "inverting";

export interface PwmUpdate {
  sequence: number;
  boardTimestampUs: number;
  pin: number;
  dutyValue: number;
  resolutionBits: number;
  outputMode: PwmOutputMode;
  timerNumber: number;
  timerChannel: PwmTimerChannel;
  waveformMode: PwmWaveformMode;
  outputPolarity: PwmOutputPolarity;
  timerClockHz: number;
  prescaler: number;
  top: number;
  compareValue: number;
  counterValue: number;
  controlA: number;
  controlB: number;
  periodNs: number | null;
  highTimeNs: number | null;
  lowTimeNs: number | null;
  frequencyMillihz: number | null;
  dutyPpm: number;
}

export interface PwmBatch {
  updates: PwmUpdate[];
  coalescedUiUpdates: number;
}

export interface BoardDescriptor {
  boardType: "arduinoUnoR3";
  firmwareVersion: {
    major: number;
    minor: number;
    patch: number;
  };
  capabilities: number;
  resetCause:
    | "unknown"
    | "powerOn"
    | "external"
    | "brownOut"
    | "watchdog"
    | "software";
  nominalLogicMv: number;
}

export interface ProtocolDiagnostic {
  category:
    | "corruptFrame"
    | "missingPackets"
    | "duplicatePacket"
    | "outOfOrderPacket"
    | "boardReset"
    | "queuePressure";
  message: string;
}

export interface DiagnosticEntry extends ProtocolDiagnostic {
  id: number;
  receivedAt: Date;
}
