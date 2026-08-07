# ADR 0008: Board LEDs Follow Observed Hardware Sources

## Status

Accepted for the Milestone 3 working tree.

## Decision

- The `L` indicator follows validated ASV GPIO events for D13. D13 events are
  preserved under queue pressure so an ordinary Blink sketch keeps its cadence.
- `TX` follows bytes the Uno USB bridge transmits to the desktop.
- `RX` follows bytes the desktop transmits to the Uno USB bridge. It remains off
  when the application has not sent any bytes.
- Each real serial activity observation holds the corresponding indicator on for
  100 ms. Further traffic extends the pulse. This intentionally keeps burst
  boundaries readable on a desktop display; it is an activity visualization,
  not an electrical reconstruction of the ATmega16U2 LED drive waveform.
- Mock Mode does not invent USB activity.

## Why

The physical `L` LED belongs to the ATmega328P D13 signal, but the physical
`TX` and `RX` LEDs belong to the separate ATmega16U2 USB-to-serial bridge. Using
separate sources keeps the drawing honest: the application displays observed
activity instead of running an unrelated animation.

## Limits

The `L` indicator can only show D13 changes reported through ASV instrumentation.
The `RX` indicator remains off in the current receive-only ASV session because
the desktop does not transmit serial payload bytes to the board. Opening the
port and changing DTR are not RX byte activity.
The desktop display is not an electrical measurement and cannot reproduce
transitions faster than the display refresh rate.
