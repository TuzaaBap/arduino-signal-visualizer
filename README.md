# Arduino Signal Visualizer

Arduino Signal Visualizer is a cross-platform desktop application that turns
instrumentation events from a real Arduino Uno R3 into understandable live
visuals.

Version 1 observes operations made through the `ASV` Arduino library. It is not
a simulator and it does not passively inspect unmodified sketches.

## Milestone 1

The first milestone provides an end-to-end digital GPIO path:

1. An instrumented sketch calls `ASV.digitalWrite`.
2. The Arduino library sends a checksummed COBS frame over USB serial.
3. Rust validates and decodes the frame.
4. React receives typed GPIO events and updates an interactive Uno view.

A deterministic **Mock Mode** exercises the same typed event path when hardware
is unavailable.

## Development

See [docs/development-setup.md](docs/development-setup.md) for prerequisites and
commands. Protocol details are in
[protocol/specification.md](protocol/specification.md).

## License

MIT

