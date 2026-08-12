# ASV wire protocol v2

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

Protocol-v2 frames use a leading `0x00`, a COBS-encoded packet, and a trailing
`0x00`. Normal sketch Serial bytes remain outside these framed regions.

| Offset | Size | Field |
| --- | ---: | --- |
| 0 | 4 | ASCII magic `ASV2` |
| 4 | 1 | Protocol version (`2`) |
| 5 | 1 | Packet type |
| 6 | 2 | Sequence number |
| 8 | 4 | Board timestamp in microseconds |
| 12 | 2 | Payload length |
| 14 | N | Payload |
| 14 + N | 2 | CRC-16/CCITT-FALSE |

The CRC covers the header and payload, but not the CRC bytes, COBS overhead, or
zero delimiter. Parameters are polynomial `0x1021`, initial value `0xFFFF`, no
reflection, and no final XOR.

The maximum decoded packet is 128 bytes. A receiver discards an overlong frame
until the next delimiter.

## Common fields

`protocol version` is `2`. The receiver retains protocol-v1 decode support for
existing firmware, but new firmware emits v2.

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

Capability bits are: bit 0 digital GPIO, bit 1 ADC, bit 2 PWM, and bit 3
separated user Serial. Bits 4 and 5 remain reserved for future hardware-
validated bus instrumentation.

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

### `0x11` ADC sample

ADC sample payload schema version 1:

| Payload offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | ADC event schema version (`1`) |
| 1 | 1 | Analog channel (`0` through `5` for Uno A0-A5) |
| 2 | 2 | Raw ADC count |
| 4 | 1 | ADC resolution in bits |
| 5 | 1 | Reference mode |
| 6 | 2 | Reference voltage in millivolts |

Supported resolution values are 8, 10, 12, 14, and 16 bits. The raw count must
not exceed `(2^resolution_bits) - 1`.

Reference modes are `0` default supply, `1` internal, and `2` external. A zero
reference voltage is permitted only for an external reference whose voltage is
unknown; the desktop must then omit calculated voltage. Non-zero reference
voltages must be no greater than 6,000 mV.

The board timestamp is carried in the common header. The Uno never transmits a
floating-point voltage. Consumers calculate voltage from validated integer
metadata as:

```text
voltage_mV = raw_count * reference_mV / full_scale_count
```

The displayed value is an estimate based on the declared reference, not a
calibrated or oscilloscope-grade measurement.

### `0x12` PWM write

PWM write payload schema version 2:

| Payload offset | Size | Field |
| --- | ---: | --- |
| 0 | 1 | PWM event schema version (`2`) |
| 1 | 1 | Arduino digital pin |
| 2 | 2 | Requested duty count |
| 4 | 1 | Duty resolution in bits |
| 5 | 1 | Output mode |
| 6 | 1 | Timer number |
| 7 | 1 | Timer channel (`0` A, `1` B) |
| 8 | 1 | Waveform mode |
| 9 | 1 | Output polarity |
| 10 | 4 | Timer source clock in hertz |
| 14 | 2 | Timer prescaler |
| 16 | 2 | Timer TOP value |
| 18 | 2 | Output compare register value |
| 20 | 2 | Timer counter snapshot |
| 22 | 1 | Raw timer control register A |
| 23 | 1 | Raw timer control register B |

For the Uno profile, only hardware PWM pins D3, D5, D6, D9, D10, and D11 are
accepted, and the declared resolution must be 8 bits. Duty counts therefore
range from 0 through 255.

Output modes are `0` constant low, `1` hardware PWM, and `2` constant high.
Duty 0 must use constant low, duty 255 must use constant high, and duty 1
through 254 must use hardware PWM. Output polarity is `0` disconnected, `1`
non-inverting, or `2` inverting. Constant endpoints require a disconnected
timer output; hardware PWM requires a connected polarity.

Waveform modes are `1` Fast PWM, `2` phase-correct PWM, and `3` phase-and-
frequency-correct PWM. Rust validates timer/pin/channel mapping, clock,
prescaler, TOP, compare, counter, polarity, CRC, and sequence before deriving
the configured waveform.

For Fast PWM:

```text
period_ticks = TOP + 1
non_inverting_high_ticks = compare
```

For dual-slope phase-correct modes:

```text
period_ticks = 2 * TOP
non_inverting_high_ticks = 2 * compare
```

The desktop derives frequency, period, HIGH time, LOW time, and duty using the
reported timer source clock and prescaler. Integer wire and backend units avoid
AVR floating-point calculations. The rendered square wave is the configured
MCU timer waveform, not an electrical voltage capture; oscillator tolerance,
loading, noise, and edge shape still require measurement hardware.

ASV does not report non-PWM pins as PWM and does not model Arduino's digital
fallback behavior on those pins.

Packet type IDs `0x13` and `0x14` are reserved for future bus instrumentation.
Version 0.6 does not emit or decode them. They must not be activated until the
firmware, protocol, desktop behavior, and representative hardware have passed
physical validation.

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
