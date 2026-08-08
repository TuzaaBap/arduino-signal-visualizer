# Automatic Arduino library validation

Date: 2026-08-09

Status: `0.5.0` release candidate physically validated; not yet released.

## Objective

Prove that an Arduino Uno sketch can activate ASV with only:

```cpp
#include <ASVInstrumented.h>

void setup() {
}

void loop() {
}
```

The sketch must not call `ASV.begin()`, `ASV.attach()`, or `ASV.service()`, and
normal Arduino `Serial` operations must remain available.

## Target

- Board: Arduino Uno on COM6
- FQBN: `arduino:avr:uno`
- USB VID/PID: `2341:0043`
- MCU signature: `1E 95 0F` (ATmega328P)
- Upload protocol: Arduino/STK500v1 at 115200 baud

## BareMinimum

- Latest source rebuilt before upload.
- Flash: 3,752 bytes of 32,256 (11%).
- SRAM: 276 bytes of 2,048 (13%).
- HEX SHA-256:
  `DEB679798C300B278134D6D271E1471BC7577F04E0F904C8B7B3F010AEC758BA`.
- One upload attempt succeeded and AVRdude verified all flash bytes.
- Desktop lifecycle:
  `waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.
- Board descriptor: Arduino Uno R3, firmware 0.4.0, capabilities `0x000F`.
- CRC failures: 0.
- Protocol diagnostics: 0.
- Dropped-packet warnings: 0.

## TransparentSerialDemo

The example uses ordinary `Serial.begin`, `Serial.print`, `Serial.read`, and
`Serial.write` calls. It contains no ASV lifecycle call.

- Latest source rebuilt before upload.
- Flash: 4,596 bytes of 32,256 (14%).
- SRAM: 375 bytes of 2,048 (18%).
- HEX SHA-256:
  `7DEC38C410AFC4ED7232FC59628F16254B6E7BA14A2745E14046FE47B34F70C9`.
- One upload attempt succeeded and AVRdude verified all flash bytes.
- 20-second desktop run, including automatic reconnect:
  - GPIO updates: 15.
  - D13 HIGH observations: 7.
  - D13 LOW observations: 8.
  - Separated user Serial bytes: 342.
  - Dropped user Serial bytes: 0.
  - CRC failures: 0.
  - Protocol diagnostics: 0.
  - Dropped-packet warnings: 0.
- Focused UI synchronization run:
  - GPIO updates: 11.
  - UI acknowledgements: 11.
  - Latest UI state matched the backend state.
- Raw user UART echo marker `ASV_USER_SERIAL_ECHO_20260809` was returned
  successfully while ASV telemetry used the same UART.

### 0.5.0 release-candidate revalidation

After synchronizing the desktop, firmware hello, and Arduino library versions,
the exact release-candidate source was rebuilt and uploaded once successfully.

- Firmware: 0.5.0.
- Flash: 4,596 bytes of 32,256 (14%).
- SRAM: 375 bytes of 2,048 (18%).
- HEX SHA-256:
  `4C1DEBA673F870DCDC331D55324A8153227AF389188AE157346D3B50D4D651D6`.
- 23-second desktop 0.5.0 run, including automatic reconnect:
  - Connection lifecycle:
    `waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.
  - GPIO updates: 19.
  - D13 HIGH observations: 9.
  - D13 LOW observations: 10.
  - UI acknowledgements: 19.
  - Latest UI state matched the backend state.
  - Separated user Serial bytes: 488.
  - Dropped user Serial bytes: 0.
  - CRC failures: 0.
  - Protocol diagnostics: 0.
  - Dropped-packet warnings: 0.
- Release Arduino ZIP SHA-256:
  `C5FB2CB60B239D26780D641A37342100B906B8951B82F79D8F0E937854C0EF23`.

## Software and packaging checks

- BareMinimum, GPIO, ADC, PWM, and transparent Serial examples compile with
  Arduino CLI warnings enabled.
- The explicit advanced API compiles without lifecycle symbol conflicts.
- All four PlatformIO targets compile.
- A deterministic library ZIP installs into an isolated Arduino user directory.
- All five examples compile from that installed ZIP.
- Shared GPIO, ADC, and PWM C++ protocol vectors pass.
- No packaged example calls `ASV.begin`, `ASV.attach`, or `ASV.service`.

## Hardware boundary

ASV preserves normal text Serial traffic by framing and separating telemetry in
the desktop application. It still consumes the Uno's only hardware UART. A
separate external binary peripheral protocol on D0/D1 cannot safely share that
wire unless it also participates in an explicit framing scheme.
