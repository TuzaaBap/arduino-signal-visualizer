import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import type {
  BoardDescriptor,
  ConnectionStatus,
  GpioBatch,
  GpioUpdate,
  ProtocolDiagnostic,
  SerialPortDescriptor,
} from "../domain/types";

export interface BackendHandlers {
  onConnectionStatus: (status: ConnectionStatus) => void;
  onBoardInfo: (board: BoardDescriptor) => void;
  onGpioBatch: (batch: GpioBatch) => void;
  onDiagnostic: (diagnostic: ProtocolDiagnostic) => void;
}

export function backendAvailable(): boolean {
  return isTauri();
}

export async function subscribeToBackend(
  handlers: BackendHandlers,
): Promise<UnlistenFn> {
  const unlisten = await Promise.all([
    listen<ConnectionStatus>("asv://connection-status", (event) =>
      handlers.onConnectionStatus(event.payload),
    ),
    listen<BoardDescriptor>("asv://board-info", (event) =>
      handlers.onBoardInfo(event.payload),
    ),
    listen<GpioBatch>("asv://gpio-batch", (event) =>
      handlers.onGpioBatch(event.payload),
    ),
    listen<ProtocolDiagnostic>("asv://protocol-diagnostic", (event) =>
      handlers.onDiagnostic(event.payload),
    ),
  ]);

  return () => {
    for (const removeListener of unlisten) {
      removeListener();
    }
  };
}

export async function listSerialPorts(): Promise<SerialPortDescriptor[]> {
  return invoke<SerialPortDescriptor[]>("list_serial_ports");
}

export async function connectSerial(
  portName: string,
  baudRate: number,
): Promise<void> {
  return invoke("connect_serial", { portName, baudRate });
}

export async function startMock(): Promise<void> {
  return invoke("start_mock");
}

export async function disconnect(): Promise<void> {
  return invoke("disconnect");
}

export async function startHardwareValidation(): Promise<boolean> {
  return invoke<boolean>("validation_start");
}

export async function acknowledgeValidationGpio(
  updates: GpioUpdate[],
): Promise<void> {
  return invoke("validation_acknowledge_gpio", { updates });
}
