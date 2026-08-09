# Beta usability and UART validation

Date: 2026-08-09

Status: development candidate physically validated, including the 30-minute
stability requirement.

## Objective

Verify that ordinary school and self-learning sketches can share the Arduino
Uno's single UART with ASV telemetry without ASV lifecycle calls, corrupted
frames, false sequence gaps, starvation, or application UI changes.

## UART design

- Default transport: 115200 baud, 8N1, or 11,520 theoretical bytes per second.
- Encoded frame sizes: GPIO 23 bytes, ADC 27 bytes, PWM 43 bytes.
- GPIO, ADC, and PWM use fixed latest-state slots with round-robin service.
- Sequence numbers advance only after a complete frame is accepted by the UART.
- Normal text `Serial` traffic remains raw and separate from CRC-protected ASV
  telemetry.
- A one-second board hello beacon supports reconnect when DTR reset is missed.
- The desktop clears stale USB input before a deliberate DTR reset and ignores
  repeated in-sequence hello beacons after connection.

This design guarantees ordered, gap-free telemetry while the generated data
fits the physical UART capacity. When a sketch exceeds that capacity, ASV keeps
the newest state for each signal; it does not claim to capture every electrical
edge like a logic analyzer.

## Mixed-stream hardware tests

Both sketches used normal Arduino calls and shared user `Serial` output with
ASV telemetry on COM6.

| Test | Time | GPIO | ADC | PWM | User Serial | Result |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Safe mixed stream | 34.2 s | 288 | 5,639 | 1,425 | 9,724 bytes | Passed |
| Deliberate overload | 26.1 s | 558 | 2,762 | 1,958 | 3,095 bytes | Passed |

The safe test included forced disconnect/reconnect and UI synchronization for
GPIO, ADC, and PWM. The overload test exceeded the event-generation capacity;
every stream continued to progress, both D13 levels were observed, and the
estimated delivered stream was 6,684 bytes per second.

For both tests:

- CRC failures: 0.
- Protocol diagnostics: 0.
- Dropped-packet warnings: 0.
- Dropped user-Serial bytes: 0.
- Application crashes: 0.
- Disconnect/reconnect failures: 0.

## School sketch matrix

Fifteen beginner-style sketches were rebuilt, uploaded once, connected to the
desktop application, and checked against their expected telemetry.

| Sketch | Coverage | Result |
| --- | --- | --- |
| Empty setup and loop | Automatic startup and board discovery | Passed |
| Blink | D13 HIGH/LOW | Passed |
| Two LEDs | D12 and D13 alternating | Passed |
| Traffic lights | D8, D9, and D10 sequencing | Passed |
| All digital pins | D2 through D13 together | Passed |
| Running light | D2 through D13 sequentially | Passed |
| Input pull-up | D2 read and D13 output | Passed |
| Serial counter | Normal `Serial.print` | Passed |
| Serial and Blink | GPIO and user Serial together | Passed |
| Analog read | A0 and user Serial | Passed |
| Analog threshold | A0, D13, and user Serial | Passed |
| PWM fade | D9 duty sweep | Passed |
| All PWM pins | D3, D5, D6, D9, D10, and D11 | Passed |
| Mixed dashboard | A0, D9 PWM, D13, and user Serial | Passed |
| Digital sweep | D2 through D13 at 25 ms steps | Passed |

Aggregate observed traffic was 507 GPIO updates, 63 ADC samples, 187 PWM
updates, and 824 user-Serial bytes. All 15 application runs had zero CRC
failures, protocol diagnostics, dropped-packet warnings, and user-Serial drops.

All sketches compile with Arduino CLI warnings enabled. They use 11–16% of Uno
flash and 15–16% of Uno SRAM. None calls `ASV.begin()`, `ASV.attach()`, or
`ASV.service()`.

The repeatable Windows hardware runner is
`scripts/validate-school-sketches.ps1`. Raw reports are written under
`work/hardware-validation/` and are intentionally excluded from release
packages.

## Continuous stability validation

The safe mixed-stream sketch ran on the Uno for 30 minutes while the desktop
application received GPIO, ADC, PWM, and ordinary user-Serial traffic. A forced
disconnect/reconnect was performed halfway through the run.

| Measurement | Result |
| --- | ---: |
| Physical test window | 30 min 0.8 s |
| Final report elapsed time | 31 min 18.8 s |
| GPIO updates | 18,727 |
| ADC samples | 367,610 |
| PWM updates | 93,623 |
| User-Serial data | 671,814 bytes |
| CRC failures | 0 |
| Protocol diagnostics | 0 |
| Dropped-packet warnings | 0 |
| Dropped user-Serial bytes | 0 |
| Maximum ADC UI buffer | 180 samples |
| Maximum PWM UI buffer | 180 samples |

The application stayed alive for the complete test and recovered through
`waiting -> connected -> disconnected -> waiting -> connected`. GPIO, ADC, and
PWM UI acknowledgements all matched backend state. The firmware occupied 5,364
bytes of flash (16%) and 362 bytes of SRAM (17%), leaving 1,686 bytes for local
variables.

Memory was sampled 61 times over 30 minutes. Working set started at 26.40 MiB,
ended at 27.36 MiB, and remained between 27.36 and 27.39 MiB during the final ten
minutes. Private memory started at 5.46 MiB, ended at 6.93 MiB, and remained
between 6.92 and 6.95 MiB during the final ten minutes. This plateau, together
with the fixed 180-sample UI limits, shows no unbounded growth during the test.

The final application report and memory samples are retained locally under
`work/hardware-validation/` and are intentionally excluded from release
packages.

## Remaining boundaries

- D0/D1 remain reserved for the shared USB UART.
- A sketch that continuously saturates normal `Serial` can consume the entire
  physical link; ASV cannot create bandwidth that the ATmega328P UART lacks.
- Arbitrary unframed binary user data containing zero delimiters still requires
  the planned strict-binary mode.
- UI waveform views describe instrumented MCU state; they are not electrical
  oscilloscope or logic-analyzer captures.
