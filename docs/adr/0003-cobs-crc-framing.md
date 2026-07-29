# ADR 0003: COBS framing with CRC-16

- Status: Accepted
- Date: 2026-07-29

## Decision

Packets use a zero-delimited COBS frame. The decoded packet ends with a
CRC-16/CCITT-FALSE covering its header and payload.

## Reason

A delimiter lets the receiver recover after partial or damaged data. COBS
guarantees the delimiter does not appear inside a frame, while the CRC detects
bit errors that framing alone cannot detect.

## Consequence

Both firmware and Rust implementations must pass the shared byte vectors before
wire changes are accepted.

