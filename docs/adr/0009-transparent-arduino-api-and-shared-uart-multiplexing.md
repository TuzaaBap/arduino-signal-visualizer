# ADR 0009: Transparent Arduino API and Shared-UART Multiplexing

## Status

Accepted and physically validated on an Arduino Uno R3 on 2026-08-08. The
automatic instrumented lifecycle was physically validated on 2026-08-09.

## Decision

- Protocol v2 ASV packets use a leading zero delimiter, COBS-encoded `ASV2`
  magic and typed packet body, CRC-16, and a trailing zero delimiter.
- The Rust transport decoder continues accepting protocol-v1 ASV-only frames.
- Ordinary sketch `Serial.print()` bytes remain raw and are delivered to a
  separate, bounded Serial Monitor stream. They are never interpreted as ASV
  events unless they form a complete signed and CRC-valid ASV v2 frame.
- Desktop-to-board Serial Monitor input remains raw user data. ASV does not send
  control packets in that direction in this version.
- `ASVInstrumented.h` redirects ordinary sketch GPIO, ADC, and PWM calls to ASV
  wrappers. The core `Serial` object remains unchanged.
- The same header redirects the sketch lifecycle to weak library hooks. The
  library starts the UART at 115200 baud, runs the user's ordinary `setup()`,
  attaches ASV at the resulting Serial configuration, and services pending
  startup telemetry around each ordinary `loop()` call. User sketches do not
  call ASV lifecycle methods.
- ASV telemetry checks the Uno hardware transmit-buffer capacity and drops an
  event instead of blocking the sketch. Sequence gaps expose dropped telemetry.

## Why

The Uno exposes one hardware UART through both D0/D1 and its USB-to-serial
bridge. A second physical channel is unavailable, so ASV telemetry and user
serial output require logical separation on the same byte stream. Preserving
the core `Serial` object keeps normal Arduino sketches and user input behavior
familiar while explicit v2 frame boundaries let the desktop demultiplex output.

## Bounds and limits

- The backend user-serial buffer is 8 KiB and the frontend buffer is 16 KiB;
  both drop oldest/excess data with explicit counters.
- Instrumentation redirects apply to the sketch translation unit. Third-party
  libraries compiled separately are not automatically instrumented.
- `ASVInstrumented.h` should be included after third-party headers so its
  Arduino API and lifecycle macros do not rewrite their declarations.
- Calling normal `Serial.begin(baud)` in the user's `setup()` overrides the
  automatic 115200 baud default. ASV does not replace `Serial`, but it must
  share the Uno's single hardware UART because no second USB data channel
  exists.
- Perfect separation of arbitrary binary user data is impossible on one wire
  unless that data is also framed. A later strict-binary mode may provide an
  opt-in framed Serial proxy.
- D0/D1 remain electrically shared with the USB UART.
