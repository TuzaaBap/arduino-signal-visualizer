import { describe, expect, it } from "vitest";

import {
  EMPTY_USER_SERIAL_STATE,
  USER_SERIAL_BUFFER_CAPACITY,
  appendUserSerial,
} from "./user-serial-store";

describe("user serial store", () => {
  it("preserves normal serial bytes and backend drop counts", () => {
    const state = appendUserSerial(EMPTY_USER_SERIAL_STATE, {
      bytes: [72, 101, 108, 108, 111],
      droppedBytes: 3,
    });

    expect(state).toEqual({
      bytes: [72, 101, 108, 108, 111],
      receivedBytes: 5,
      droppedBytes: 3,
    });
  });

  it("keeps only the newest bytes when the UI buffer reaches its bound", () => {
    const state = appendUserSerial(
      {
        bytes: Array.from({ length: USER_SERIAL_BUFFER_CAPACITY }, () => 1),
        receivedBytes: USER_SERIAL_BUFFER_CAPACITY,
        droppedBytes: 0,
      },
      { bytes: [2, 3], droppedBytes: 0 },
    );

    expect(state.bytes).toHaveLength(USER_SERIAL_BUFFER_CAPACITY);
    expect(state.bytes.slice(-2)).toEqual([2, 3]);
    expect(state.droppedBytes).toBe(2);
  });
});
