# Milestone 2 plan: ADC

This plan defined the ADC vertical slice and was approved for implementation.
The completed results are recorded in `docs/milestone-2-validation.md`.

## Recommended architecture

The firmware should wrap Arduino's `analogRead()` in the same way Milestone 1
wraps GPIO: call the real Arduino function, return its result unchanged, and
emit the observed sample afterward. This preserves normal sketch behavior and
keeps instrumentation optional.

The wire protocol should add a typed `AnalogSample` packet containing:

- Arduino analog pin/channel.
- Raw ADC code.
- ADC resolution in bits.
- Board timestamp and packet sequence from the existing header.
- Reference-source identifier and an optional explicitly configured reference
  voltage.

Raw ADC counts are the canonical value. The desktop must not present a computed
voltage as measured truth unless the reference voltage is explicitly known.
The Uno's nominal 5 V supply is not a calibrated AREF measurement.

Rust remains the only desktop layer that parses packet bytes. It should validate
Uno channel limits and the 10-bit code range before emitting a typed event.
The connection worker should coalesce only display updates; future recording
must branch before coalescing so it can retain every sample.

React should keep a fixed-size ring buffer per analog channel. A bounded buffer
prevents memory growth, while a 30 Hz presentation cadence keeps rendering
independent of sample rate.

## Task breakdown

1. **Protocol contract**
   - Specify `AnalogSample` layout and reference semantics.
   - Add shared binary vectors and Rust encode/decode/error tests.
   - Milestone gate: protocol tests and workspace build pass.

2. **Firmware API**
   - Add `ASV.analogRead(pin)` with standard Arduino return semantics.
   - Add explicit ADC-reference metadata configuration.
   - Add an `AdcDemo` example for A0 without modifying `GpioDemo`.
   - Milestone gate: Uno firmware compiles with flash/SRAM budgets recorded.

3. **Typed desktop pipeline**
   - Decode validated analog events in Rust.
   - Add bounded analog batching and deterministic Mock Mode samples.
   - Keep GPIO and ADC event types independent but share connection lifecycle.
   - Milestone gate: Rust and mock tests pass with no queue regressions.

4. **ADC user interface**
   - Add per-channel raw-count cards and fixed-size trend plots.
   - Show voltage only when a reference value is explicitly available.
   - Add reducer, accessibility, and bounded-buffer tests.
   - Milestone gate: TypeScript, frontend tests, and production bundle pass.

5. **Hardware validation**
   - Validate A0 at ground, a stable midpoint, and the selected reference.
   - Confirm raw-code range, monotonic response, UI agreement, reconnect, CRC,
     packet-loss, and queue counters.
   - Repeat the 30-minute memory/stability observation.
   - Record results before declaring Milestone 2 complete.
