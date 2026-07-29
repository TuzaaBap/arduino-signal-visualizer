# ADR 0004: Compiled-in extension points for v1

- Status: Accepted
- Date: 2026-07-29

## Decision

Board decoders and protocol feature handlers use Rust interfaces and registries,
but are compiled into the application in v1.

## Reason

The code still has clean extension points without introducing the signing,
compatibility, and security problems of loading third-party native code.

## Consequence

Adding a board requires a new module and application release. Runtime plugins
can be designed later against proven interfaces.

