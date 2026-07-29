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
