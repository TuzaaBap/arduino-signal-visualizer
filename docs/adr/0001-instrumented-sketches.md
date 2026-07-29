# ADR 0001: Instrumented sketches in v1

- Status: Accepted
- Date: 2026-07-29

## Decision

Version 1 observes Arduino operations made through the `ASV` library. It does
not modify the Arduino core and does not claim to passively inspect unmodified
sketches.

## Reason

An Uno cannot report every physical bus transition over its own USB serial
connection without cooperation from the sketch or separate analyser hardware.
An explicit library is predictable, teachable, and safe.

## Consequence

Beginners change calls such as `digitalWrite(13, HIGH)` to
`ASV.digitalWrite(13, HIGH)`. Future passive analysis requires separate
hardware and is outside v1.

