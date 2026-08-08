# Protocol v2 transparent Serial validation

Date: 2026-08-08

Status: physically validated; changes remain uncommitted pending product review.

## Hardware and upload

- Board: Arduino Uno R3 on `COM6`, USB VID/PID `2341:0043`, serial
  `7583435373035140D122`.
- MCU signature: ATmega328P `0x1e950f`.
- Firmware and desktop version: `0.4.0`.
- Firmware: `firmware/examples/TransparentSerialDemo/TransparentSerialDemo.ino`.
- PlatformIO environment: `atmelavr@5.3.0`, board `uno`, Arduino framework.
- Uploader: avrdude 6.3 using the `arduino` protocol.
- Upload result: first and only attempt succeeded; all 3,114 flash bytes were
  written and verified.
- Flash: 3,114 of 32,256 bytes (9.7%).
- SRAM: 314 of 2,048 bytes (15.3%).
- Uploaded HEX SHA-256:
  `8AB3B510348DB5DD35666E0CA83E49B74FA2790F4F9E17CFCD6A2E68AD541181`.

## Shared-UART functional validation

- The optimized desktop release automatically connected at 115200 baud and
  reported an Arduino Uno R3, firmware `0.4.0`, and capability mask `15`.
- Normal `Serial.print()` lines appeared only in the Serial Monitor while ASV
  D13 events continued driving the instrumented GPIO path.
- The UI sent `ASV-ECHO-VALIDATION-4821`; the Uno received and echoed the exact
  line, and the Serial Monitor displayed it.
- A ten-line `ASV-BURST-00` through `ASV-BURST-09` test returned every line in
  order while D13 telemetry continued between the lines.
- D13 produced alternating HIGH and LOW observations. The final stability
  snapshot contained 1,840 GPIO updates: 919 HIGH and 921 LOW. The two-count
  difference is explained by startup/reconnect boundary observations.
- Frontend GPIO delivery remained active, with 1,839 UI acknowledgements in the
  final snapshot.

## Thirty-minute stability gate

The authoritative run used the optimized desktop `0.4.0` release with the real
Serial Monitor and board UI mounted. It ran for 1,844.528 seconds and performed
one controlled disconnect/reconnect after 60 seconds.

- Connection history:
  `waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.
- User Serial bytes separated by the backend: 56,744.
- Dropped backend user Serial bytes: 0.
- CRC failures: 0.
- Dropped-packet/queue warnings: 0.
- Protocol diagnostics: 0.
- Application crashes: 0; the process remained responsive.
- Memory samples: 61 at approximately 30-second intervals.
- Private memory range: 5.305-6.008 MiB, ending below the initial sample.
- Working-set range: 26.504-26.945 MiB.
- The frontend Serial history exercised its fixed 16 KiB bound while continuing
  to display the newest text. The backend delivery buffer remained bounded at
  8 KiB and reported no loss.

## Automated verification

- Rust: 47 tests passed (43 protocol and 4 desktop/backend).
- Frontend: 21 tests passed across 7 test files.
- Rust formatting and Clippy with warnings denied passed.
- TypeScript checking and the Vite production build passed.
- Native C++ shared-vector tests passed for GPIO, ADC, and PWM protocol-v2
  frames.
- PlatformIO successfully compiled the transparent Serial Uno example.

## Known limits

- Transparent mode is intended for ordinary text Serial traffic. Arbitrary
  binary user streams containing framing delimiters require the planned strict
  binary mode to guarantee separation.
- `ASVInstrumented.h` redirects Arduino calls in the sketch translation unit.
  Separately compiled third-party libraries are not automatically instrumented.
- D0/D1 remain electrically shared with the Uno USB serial bridge.
- ASV telemetry yields to user Serial traffic instead of blocking the sketch.
  A saturated transmit path may therefore create a visible ASV sequence gap;
  none occurred during this validation.
