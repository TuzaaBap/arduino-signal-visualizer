# Milestone 3 PWM validation

Date: 2026-08-03

Status: revised timer-derived implementation and three-minute Uno smoke demo
pass; electrical comparison and 30-minute stability validation pending.

Milestone 3 is not approved for commit, tag, or release by this document.

## Revised scope

- `ASV.analogWrite(pin, value)` accepts only D3, D5, D6, D9, D10, and D11 and
  integer values from 0 through 255.
- PWM protocol schema 2 reports requested count plus actual timer/channel,
  waveform mode, polarity, source clock, prescaler, TOP, compare, counter, and
  raw control registers.
- Rust rejects impossible metadata and derives frequency, period, HIGH time,
  LOW time, and duty using integer units.
- The frontend renders a rectangular `Configured MCU waveform` with a selectable
  1.000, 2.500, 5.000, or 10.000 ms time window.
- Duty history remains bounded for recording and validation but is never drawn
  as the electrical waveform.
- GPIO and ADC paths remain independent and unchanged.

## Current software verification

| Gate | Result |
| --- | --- |
| Rust protocol tests | Passed: 36 |
| Rust backend tests | Passed: 3 |
| Frontend store/waveform tests | Passed: 11 |
| Total automated tests | Passed: 50 |
| TypeScript check | Passed |
| Frontend production build | Passed |
| Revised PWM firmware build | Passed |
| Native C++ shared-vector test | Passed |
| GPIO and ADC firmware regression builds | Passed |
| Browser visual/interaction check | Passed at 900, 1,100, and 1,400 px widths; SVG containment, 12 px panel clearance, pin gating, timebase, and console health verified |

PWM protocol tests include malformed length, unsupported schema, invalid pin,
resolution and duty, endpoint-mode mismatch, timer/channel mismatch, waveform
mode, polarity, clock, prescaler, TOP, compare, counter, CRC, sequence, shared
Rust/C++ vector, and exact Fast/phase-correct timing equations.

## Revised firmware size

| Target | Flash | SRAM |
| --- | ---: | ---: |
| GPIO regression | 3,138 bytes (9.7%) | 236 bytes (11.5%) |
| ADC regression | 3,006 bytes (9.3%) | 230 bytes (11.2%) |
| PWM demo | 3,530 bytes (10.9%) | 249 bytes (12.2%) |

## Superseded physical evidence

The earlier schema-1 `PwmDemo` upload and 30-minute run passed transport,
reconnect, bounded-memory, CRC, and UI-delivery checks. It did not transmit timer
registers and displayed requested-duty history. Those results do not validate
schema 2 and cannot be used as the revised Milestone 3 release gate.

## Revised schema-2 Uno smoke demo

The revised `PwmDemo` was rebuilt immediately before one approved upload to the
Arduino Uno R3 on COM6. AVRDude identified the ATmega328P signature as
`0x1e950f`, wrote 3,530 bytes, and verified flash without a retry.

The production application connected automatically and displayed the PWM tab
on screen for a timed 180.3-second interval. The application remains open after
the demo for manual inspection.

| Gate | Result |
| --- | --- |
| Firmware | Passed: 0.3.0, capabilities 7 |
| PWM updates | Passed: 3,241 |
| UI acknowledgements | Passed: 784 |
| PWM pins observed | Passed: all six |
| Per-pin duty range | Passed: 0 through 255 |
| UI/backend match | Passed |
| Maximum UI buffer | Passed: 180 states per pin |
| Protocol diagnostics | Passed: 0 |
| CRC failures | Passed: 0 |
| Dropped-packet warnings | Passed: 0 |
| Application crash | Passed: none |
| Private memory | Stable: 6.49 MiB to 6.27 MiB; range 6.23–6.49 MiB |

Live timer metadata showed D5/D6 Fast PWM at 976.563 Hz and the phase-correct
pins at 490.196 Hz with the expected integer HIGH/LOW timing. This smoke demo
does not replace the required 30-minute stability run or external electrical
comparison.

## Required configured-timing validation

For D5/D6 Fast PWM with the standard Uno core:

| Count | Output | Duty | Period | HIGH | LOW | Frequency |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 0 | Constant LOW | 0.000% | Not periodic | — | — | No carrier |
| 64 | PWM | 25.000% | 1.024 ms | 256 µs | 768 µs | 976.563 Hz |
| 128 | PWM | 50.000% | 1.024 ms | 512 µs | 512 µs | 976.563 Hz |
| 191 | PWM | 74.609% | 1.024 ms | 764 µs | 260 µs | 976.563 Hz |
| 255 | Constant HIGH | 100.000% | Not periodic | — | — | No carrier |

For D3/D9/D10/D11 phase-correct PWM with the standard Uno core:

| Count | Output | Duty | Period | HIGH | LOW | Frequency |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 0 | Constant LOW | 0.000% | Not periodic | — | — | No carrier |
| 64 | PWM | 25.098% | 2.040 ms | 512 µs | 1.528 ms | 490.196 Hz |
| 128 | PWM | 50.196% | 2.040 ms | 1.024 ms | 1.016 ms | 490.196 Hz |
| 191 | PWM | 74.902% | 2.040 ms | 1.528 ms | 512 µs | 490.196 Hz |
| 255 | Constant HIGH | 100.000% | Not periodic | — | — | No carrier |

Verify that displayed TCCR, OCR, TCNT, timer/channel, prescaler, TOP, timing, and
rectangular waveform agree with the received schema-2 packet for every pin and
count.

## Required electrical comparison

Use a high-impedance logic-analyzer or oscilloscope input with common ground.
Measure D5, D9, and D3 at counts 0, 64, 128, 191, and 255. Record measured duty,
frequency, HIGH time, LOW time, absolute error, and percentage error. Pulse
structure should match the configured trace; board-clock and analyzer tolerance
must be reported rather than hidden.

ASV does not electrically measure voltage, loading, noise, rise/fall time, or
wiring faults. A configured trace must never be described as a captured trace.

## Remaining stability gate

After a rebuilt schema-2 `PwmDemo` is uploaded, run the production application
for at least 30 minutes and verify:

- no crashes or unbounded native/UI memory growth;
- zero CRC failures, invalid packets, or dropped-packet warnings;
- bounded PWM history never exceeds 180 states per pin;
- UI/backend state and derived timing match;
- disconnect detection and reconnect recovery pass.

Do not commit or tag until the revised upload, electrical matrix, and stability
run pass.
