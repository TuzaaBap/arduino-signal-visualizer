# Arduino Signal Visualizer 0.5.1 beta

This corrective beta makes beginner-style GPIO bursts reliable. Back-to-back
`digitalWrite()` calls now use a bounded per-pin pending-state buffer, and
ordinary `delay()` calls automatically give that buffer time to reach the app.
All usable Uno digital pins can therefore update together without adding ASV
calls or artificial telemetry delays to the sketch.

Instrumentation remains automatic: include `ASVInstrumented.h`, then write
ordinary Arduino `setup()` and `loop()` functions. The library records
supported GPIO, ADC, configured PWM, board LED activity, and transparent text
Serial traffic.

## Downloads

- Windows x64: use the `-setup.exe` installer; it installs for the current user
  without administrator access. The MSI is for administrator-led or managed
  all-users deployment.
- macOS Apple silicon: use the `aarch64` DMG.
- macOS Intel: use the `x86_64` DMG.
- Arduino IDE: install `ArduinoSignalVisualizer-0.5.1.zip` through
  **Sketch > Include Library > Add .ZIP Library...**.

Use desktop application 0.5.1 with Arduino library 0.5.1. The library defaults
to 115200 baud. If the sketch calls `Serial.begin`, its selected rate wins;
select that same rate in the desktop application.

## Minimal sketch

```cpp
#include <ASVInstrumented.h>

void setup() {}

void loop() {}
```

No `ASV.begin`, `ASV.attach`, or `ASV.service` call is needed. Advanced users
can retain explicit lifecycle control by including `ArduinoSignalVisualizer.h`.

## Beta signing notice

The first beta artifacts are not backed by paid public code-signing identities.
Windows SmartScreen may show an unknown-publisher warning. macOS builds use an
ad-hoc signature and must be explicitly allowed in **Privacy & Security**.
Do not describe these artifacts as signed or notarized production releases.

## Serial behavior

The library does not replace Arduino `Serial`. Normal `Serial.print()`,
`Serial.read()`, and `Serial.write()` traffic appears in the application's
Serial tab while ASV telemetry remains separately framed and CRC-protected.
User Serial traffic has priority over telemetry.

Only one desktop program can open a serial port at a time. Close Arduino IDE
Serial Monitor and other terminals before connecting the application. While
connected, use the application's Serial tab for normal sketch communication.

Transparent mode is intended for normal text streams. Arbitrary binary traffic
containing zero delimiters requires the planned strict-binary mode.

## Hardware scope

This beta supports the Arduino Uno R3 and compatible ATmega328P Uno boards.
Timer-derived PWM views show configured MCU behavior, not an electrical
oscilloscope measurement. ADC voltage uses declared reference metadata and is
not a calibrated multimeter measurement.
