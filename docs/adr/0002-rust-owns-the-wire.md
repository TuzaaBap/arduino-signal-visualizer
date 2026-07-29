# ADR 0002: Rust owns serial bytes

- Status: Accepted
- Date: 2026-07-29

## Decision

Only the Rust backend may read, frame, validate, or decode serial bytes. React
receives board-independent typed events.

## Reason

Serial data can be partial, corrupt, or arrive much faster than a browser UI
should render. Keeping it in Rust isolates unsafe input and gives one place for
validation, reconnect handling, and future recording.

## Consequence

UI components cannot depend on Uno packet layouts. A future board decoder can
produce the same GPIO event without changing the GPIO view.

