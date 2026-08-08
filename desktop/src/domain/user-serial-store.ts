import type { UserSerialBatch } from "./types";

export const USER_SERIAL_BUFFER_CAPACITY = 16 * 1024;

export interface UserSerialState {
  bytes: number[];
  receivedBytes: number;
  droppedBytes: number;
}

export const EMPTY_USER_SERIAL_STATE: UserSerialState = {
  bytes: [],
  receivedBytes: 0,
  droppedBytes: 0,
};

export function appendUserSerial(
  current: UserSerialState,
  batch: UserSerialBatch,
): UserSerialState {
  const validBytes = batch.bytes.filter(
    (byte) => Number.isInteger(byte) && byte >= 0 && byte <= 0xff,
  );
  const combined = [...current.bytes, ...validBytes];
  const overflow = Math.max(0, combined.length - USER_SERIAL_BUFFER_CAPACITY);

  return {
    bytes: overflow > 0 ? combined.slice(overflow) : combined,
    receivedBytes: current.receivedBytes + validBytes.length,
    droppedBytes: current.droppedBytes + batch.droppedBytes + overflow,
  };
}
