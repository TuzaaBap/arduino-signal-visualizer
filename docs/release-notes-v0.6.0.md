# Arduino Signal Visualizer 0.6.0

This is the first stable Arduino Uno release of Arduino Signal Visualizer. It
combines the desktop application and matching Arduino IDE library into one
validated classroom package.

## What students can see

- Digital pin direction and HIGH/LOW state for D0-D13.
- Live ADC values, calculated voltage, full-scale percentage, and bounded
  graphs for A0-A5.
- Timer-derived rectangular PWM waveforms for D3, D5, D6, D9, D10, and D11,
  including duty, frequency, period, HIGH/LOW time, and timer metadata.
- Normal `Serial.print()` output in a separate in-app Serial Monitor.
- TX, RX, and built-in D13/L LED activity on the interactive Uno board.
- Signed in-app update discovery with **Download & install**, **Skip this
  version**, and **Not now**.

Sketches need only include `ASVInstrumented.h`; normal Arduino lifecycle and
GPIO, ADC, PWM, and Serial calls remain familiar.

The library uses bounded latest-state telemetry and reserves one third of the
configured UART capacity for normal sketch Serial output and USB timing margin.
This prevents high-rate instrumentation from creating partial or reordered
protocol frames.

## Download both matching parts

Install the desktop package for your computer and
`ArduinoSignalVisualizer-0.6.0.zip` in Arduino IDE 2. Do not use GitHub's source
archive as the Arduino library.

## Validated boundary

The supported board is Arduino Uno R3/ATmega328P. This release was physically
validated with beginner sketches, mixed GPIO/ADC/PWM/Serial workloads,
disconnect/reconnect cycles, high-rate six-channel PWM traffic, bounded UI
buffers, and packet/CRC diagnostics.

I2C and SPI **instrumentation** are intentionally excluded until representative
peripheral hardware is validated. Ordinary sketches may still use Arduino's
standard `Wire` and `SPI` libraries; the Uno diagram keeps the factual pin-role
labels.

ASV reports instrumented MCU operations. It is not an oscilloscope, electrical
probe, or logic analyzer. ADC voltage is calculated from declared reference
metadata, and PWM is reconstructed from timer configuration.

## Installation security note

Updater packages are signed with the project's embedded Tauri updater key.
The installers are not yet backed by commercial Windows Authenticode or Apple
Developer ID/notarization credentials, so SmartScreen or Gatekeeper may request
confirmation. Download only from this repository and do not disable operating-
system security globally.
