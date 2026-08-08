# Arduino Signal Visualizer

Arduino Signal Visualizer is a cross-platform desktop application that turns
instrumentation events from a real Arduino Uno R3 into understandable live
visuals.

The application observes operations instrumented by the
`ArduinoSignalVisualizer` Arduino library. Sketches can keep familiar Arduino
calls such as `digitalWrite`, `analogRead`, `analogWrite`, and normal
`Serial.print` after including `ASVInstrumented.h`. The application is not a
simulator and cannot passively inspect a sketch that does not include the
library.

## Beta quick start

Each beta release contains matching desktop and Arduino library artifacts:

1. Install the Windows setup executable or the DMG matching the Mac processor.
2. In Arduino IDE 2, choose **Sketch > Include Library > Add .ZIP Library...**
   and select `ArduinoSignalVisualizer-0.4.0.zip`.
3. Open **File > Examples > ArduinoSignalVisualizer >
   TransparentSerialDemo**, select the Arduino Uno and upload.
4. Close Arduino IDE Serial Monitor, open Arduino Signal Visualizer, select the
   board port and the baud rate used by the sketch, and connect.
5. Use the application's Serial tab for the sketch's normal text input/output
   while the application is connected.

Only one desktop process can own a serial port at a time. The app does not run
in the background or open a board until the user connects. See
[docs/distribution.md](docs/distribution.md) for supported installers, beta
signing warnings, release procedure, and checksums.

## Milestone 1

The first milestone provides an end-to-end digital GPIO path:

1. An instrumented sketch calls `ASV.digitalWrite`.
2. The Arduino library sends a checksummed COBS frame over USB serial.
3. Rust validates and decodes the frame.
4. React receives typed GPIO events and updates an interactive Uno view.

A deterministic **Mock Mode** exercises the same typed event path when hardware
is unavailable.

## Milestone 2

The second milestone adds instrumented `ASV.analogRead(A0)` support and a
versioned ADC event. The Analog tab displays A0-A5 raw counts,
desktop-calculated voltage, reference metadata, percentage of full scale, and
bounded trend graphs. These trends are intentionally not presented as
oscilloscope-grade or calibrated measurements.

## Milestone 3

The third milestone adds checked `ASV.analogWrite(pin, duty)` instrumentation
for the Uno's real hardware PWM pins: D3, D5, D6, D9, D10, and D11. The firmware
captures the driving timer registers and the desktop reconstructs a rectangular
configured waveform with period, frequency, HIGH/LOW time, duty, timer counter,
compare value, and selectable time window. The trace is timer-derived rather
than an electrical voltage measurement.

## Development

See [docs/development-setup.md](docs/development-setup.md) for prerequisites and
commands. Protocol details are in
[protocol/specification.md](protocol/specification.md).

## License

MIT
