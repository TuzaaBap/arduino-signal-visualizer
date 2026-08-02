# ADR 0005: Raw ADC counts are canonical

## Status

Accepted for Milestone 2.

## Context

An Arduino Uno ADC conversion produces an integer count. Converting that count
to volts also requires the ADC resolution and the voltage actually represented
by the selected reference. The Uno's nominal 5 V supply is not a calibrated
measurement, and transmitting floating-point values would add AVR cost while
hiding the assumptions used in the conversion.

ADC traffic can also arrive faster than a desktop UI should render. Recording
and display therefore have different loss and memory requirements.

## Decision

The wire protocol carries the raw count, resolution bits, reference mode,
integer reference millivolts, channel, sequence, and board timestamp. Rust
rejects impossible combinations before producing a typed ADC event. Voltage is
calculated on the desktop from that validated metadata.

Every valid ADC event reaches the recording/validation branch. UI delivery is
separately rate-limited and bounded per channel, and React retains only the
latest 180 graph samples per channel.

## Consequences

- Recorded data preserves the original ADC observation and can be recalculated
  later if better reference calibration becomes available.
- The AVR performs no floating-point voltage calculation.
- The UI cannot consume unbounded memory or force rendering at wire speed.
- Displayed voltage is explicitly an estimate unless the supplied reference
  voltage has been measured and configured.
- Trend graphs are not presented as calibrated or oscilloscope-grade data.
