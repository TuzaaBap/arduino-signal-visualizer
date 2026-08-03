# ADR 0006: PWM timer configuration is canonical

## Status

Accepted for revised Milestone 3.

## Decision

`ASV.analogWrite()` accepts only Uno hardware PWM pins D3, D5, D6, D9, D10,
and D11 and duty counts 0 through 255. Invalid calls are rejected before
hardware changes or protocol transmission.

After a valid Arduino write, firmware captures the actual ATmega328P timer
configuration that drives the selected pin. Protocol schema 2 carries integer
timer/channel, waveform mode, polarity, clock, prescaler, TOP, compare,
counter, and raw control-register values alongside the requested count.

Rust owns timer validation and derives period, frequency, HIGH time, LOW time,
and configured duty in explicit integer units. React receives only typed,
validated timing and renders a rectangular pulse train. It never treats a
history of duty commands as the output waveform.

The interface labels the result `Configured MCU waveform`. It does not label
timer-derived timing as electrically measured.

## Consequences

- Fast PWM and dual-slope PWM use their correct, different timing equations.
- A logic analyzer should show the same steady-state pulse structure and timing
  within board-clock and analyzer tolerances.
- Duty endpoints display constant LOW or HIGH with no invented carrier.
- Timer registers remain visible for teaching and diagnosis.
- Electrical voltage, loading, noise, rise/fall time, and clock error remain
  outside software-only observation and require measurement hardware.
