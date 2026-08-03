# ADR 0007: Uno R3 vector uses semantic pin data

## Status

Accepted for the Milestone 3 working tree.

## Context

The board view must remain useful as ASV adds PWM, UART, SPI, I2C and interrupt
visualisation. Encoding alternate functions only as SVG text would allow the
drawing and application behavior to disagree. Redrawing the board for each
protocol would also create several competing Uno models.

## Decision

Keep one physically representative Uno R3 SVG and one typed semantic pin map.
The map defines the Arduino pin number, ATmega328P port, ADC channel and all
supported alternate functions. The SVG consumes that data for labels,
accessibility text, selection behavior and capability styling.

The vector includes the major physical landmarks needed for orientation, but it
is an educational diagram rather than a PCB fabrication drawing. Protocols not
yet implemented may be identified on their real pins, but their controls remain
disabled until the corresponding milestone is implemented and validated.

## Consequences

- Future protocol milestones activate existing pin definitions instead of
  redrawing or renumbering the board.
- Automated tests protect the Uno PWM, UART, SPI, I2C and interrupt mappings.
- Visual component placement can evolve without changing signal semantics.
- Manufacturing dimensions, copper routing and every passive component are out
  of scope for this application vector.
