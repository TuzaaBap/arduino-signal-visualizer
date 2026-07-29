# ASV wire protocol v1

## Purpose

The ASV protocol carries instrumentation events from a real Arduino Uno to the
desktop application. It is binary, versioned, and designed to recover from
partial or corrupt serial input.

All multi-byte integers are little-endian.

## Serial settings

- Default baud: 115200
- Data bits: 8
- Parity: none
- Stop bits: 1
- Flow control: none

## Framing

The packet below is COBS encoded and followed by one `0x00` delimiter:

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | Protocol version |
| 1 | 1 | Packet type |
| 2 | 2 | Sequence number |
| 4 | 4 | Board timestamp in microseconds |
| 8 | 2 | Payload length |
| 10 | N | Payload |
| 10 + N | 2 | CRC-16/CCITT-FALSE |

The CRC covers the header and payload, but not the CRC bytes, COBS overhead, or
zero delimiter. Parameters are polynomial `0x1021`, initial value `0xFFFF`, no
reflection, and no final XOR.

The maximum decoded packet is 128 bytes. A receiver discards an overlong frame
until the next delimiter.

## Common fields

`protocol version` is `1`. Other versions are rejected before payload decoding.

`sequence number` increases for every transmitted packet and wraps after
65535. Receivers use it to report missing, duplicate, and stale packets.

`board timestamp` is the Arduino `micros()` value and naturally wraps. It is for
relative timing, not wall-clock time.

## Packet types

### `0x01` Board hello

Sent once from `ASV.begin()` after the USB serial connection is ready.

| Payload offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | Board type (`1` = Arduino Uno R3) |
| 1 | 3 | Firmware major, minor, patch |
| 4 | 2 | Capability flags |
| 6 | 1 | Reset cause |
| 7 | 2 | Nominal logic supply in millivolts |

Capability bit 0 represents digital GPIO instrumentation.

Reset causes are `0` unknown, `1` power-on, `2` external, `3` brown-out,
`4` watchdog, and `5` software.

A hello received after a connection is established is a board-reset boundary.
The desktop clears sequence tracking and replaces the current board metadata.

### `0x10` Digital GPIO state

| Payload offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | Arduino digital pin (`0` through `13`) |
| 1 | 1 | Mode |
| 2 | 1 | Logic level |
| 3 | 1 | Observation source |

Modes are `0` input, `1` output, `2` input pull-up, and `255` unknown. Levels
are `0` low and `1` high. Sources are `0` write, `1` read, and `2` mode change.

The voltage shown for digital GPIO is a logical estimate derived from the
board's nominal supply. It is not an ADC measurement.

## Receiver fault handling

- Empty delimiters are ignored.
- An invalid COBS frame is reported and discarded.
- Frames larger than the limit are discarded through the next delimiter.
- CRC failures never reach packet-specific decoders.
- Header payload length must exactly match the decoded bytes.
- Each packet type has one exact payload length.
- Unsupported versions are reported without attempting payload decoding.
- Sequence gaps and duplicates are diagnostics, not application crashes.
- A hello packet marks reconnect/reset state even if sequence numbers appear
  valid.

## Shared vectors

Files in `protocol/test-vectors` contain complete COBS frames as lowercase hex,
including the final zero delimiter. Both Rust and native firmware tests consume
the same files.

