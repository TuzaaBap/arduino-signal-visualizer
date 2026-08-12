# v0.5.1 120-sketch compatibility validation

Date: 2026-08-12

Status: passed on a physical Arduino Uno connected to COM6.

## Purpose and scope

This matrix checks whether normal beginner Arduino code remains usable when
the sketch includes `ASVInstrumented.h`. It is a compatibility and robustness
test for the v0.5.1 application/library pair, not a claim that 120 unrelated
third-party libraries were tested.

The runner generated 120 distinct ordinary sketches across eight families,
with 15 pin/timing/data variations per family:

| Sketch family | Result |
| --- | ---: |
| Single-pin digital Blink | 15 / 15 passed |
| Multi-pin array sequencing | 15 / 15 passed |
| `INPUT_PULLUP` and `digitalRead()` | 15 / 15 passed |
| Normal `Serial.print()` and `Serial.println()` | 15 / 15 passed |
| `analogRead()` across A0-A5 | 15 / 15 passed |
| `analogWrite()` PWM fade across all PWM pins | 15 / 15 passed |
| Mixed GPIO, ADC, PWM, and normal Serial | 15 / 15 passed |
| Helper functions and `millis()` scheduling | 15 / 15 passed |

Every sketch used familiar Arduino `setup()` and `loop()` source. No sketch
called `ASV.begin()`, `ASV.attach()`, `ASV.service()`, or encoded a protocol
packet manually.

## Results

| Gate | Result |
| --- | ---: |
| Sketches generated | 120 |
| Arduino Uno compilations | 120 passed, 0 failed |
| Single-attempt physical uploads completed | 120 |
| Application/library compatibility | 120 passed, 0 failed |
| Controlled reconnect scenarios | 10 passed, 0 failed |
| GPIO updates received | 2,062 |
| ADC samples received | 709 |
| PWM updates received | 1,793 |
| Normal user-Serial bytes received | 12,846 |
| Protocol diagnostics | 0 |
| CRC failures | 0 |
| Dropped-packet warnings | 0 |
| Dropped user-Serial bytes | 0 |

All reconnect cases completed the expected sequence:
`waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.

Compiled firmware occupied 4,356-5,328 bytes of flash and 309-357 bytes of
SRAM across the matrix. These figures include ASV instrumentation and remained
well within the Uno's limits.

## Harness interruption and resolution

After physical case 47 was first uploaded, the validation harness failed to
wait for its desktop process to terminate. The surviving process held COM6,
and repeated reset/enumeration activity left the Arduino USB interface absent
from `arduino-cli`. Windows still reported the device before it was physically
reconnected. No additional upload was attempted while the port was unavailable.

The harness was corrected to stop and wait for every desktop process and to
verify that no process remains before the next upload. The Uno was physically
reconnected, case 47 was run once again, and cases 47-120 all passed. Case 47
therefore does not represent a reproducible sketch, library, or application
failure.

## Remaining compatibility boundaries

This matrix validates the instrumented Arduino API path exercised above. It
does not yet prove compatibility with every external Arduino library, direct
AVR register manipulation, interrupt-heavy code, sketches that replace timer
configuration, alternate UART baud rates, arbitrary unframed binary Serial
payloads, or code that depends on D0/D1 as independent GPIO while USB Serial is
active. Those require separate targeted matrices before making broader claims.

Raw per-sketch source, compile logs, upload logs, application reports, and the
machine-readable summary are retained locally under
`work/hardware-validation/beginner-120-20260812-070413/`.
