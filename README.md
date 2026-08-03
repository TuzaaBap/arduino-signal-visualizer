# Arduino Signal Visualizer

Arduino Signal Visualizer is a cross-platform desktop application that turns
instrumentation events from a real Arduino Uno R3 into understandable live
visuals.

The application observes operations made through the `ASV` Arduino library. It
is not a simulator and it does not passively inspect unmodified sketches.

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
