# v0.5.1 — 120 Complex Sketch Compatibility Validation

Date: 2026-08-12

Board: Arduino Uno on COM6 (`arduino:avr:uno`)

Board serial: `7583435373035140D122`

Desktop application: v0.5.1

Firmware/library protocol: v0.5.1

## Purpose

This matrix tests whether realistic, comparatively complex Arduino Uno programs can run with the ASV library while the desktop application receives instrumented GPIO, ADC, PWM, and normal user `Serial` traffic. It is a compatibility and robustness test, not proof that ASV observes every internal operation performed by third-party libraries.

## Coverage

The matrix contains 120 independently compiled and physically uploaded sketches: 10 variations in each of 12 families.

| Family | Runs | Main stress area | Corrected pass |
|---|---:|---|---:|
| Cooperative scheduler | 10 | Concurrent timed GPIO, ADC, PWM, and Serial tasks | 10/10 |
| State machine | 10 | Mode transitions, timing, ADC, and multiple outputs | 10/10 |
| C++ class controller | 10 | Encapsulated GPIO and PWM control | 10/10 |
| ADC filter | 10 | Moving-average sampling and Serial output | 10/10 |
| Event queue | 10 | Queued GPIO/PWM events and sustained Serial output | 10/10 |
| Serial parser | 10 | User Serial parsing beside ASV telemetry | 10/10 |
| PROGMEM waveform | 10 | Flash-resident lookup data and PWM updates | 10/10 |
| Multi-PWM engine | 10 | All six Uno hardware PWM channels at high update rates | 9/10 initial; failed run passed 5/5 focused repeats |
| EEPROM journal | 10 | Persistent structured data, ADC, GPIO, and Serial | 10/10 |
| Wire and SPI | 10 | I2C/SPI library coexistence with ASV | 10/10 |
| Servo and tone | 10 | Timer-using Arduino libraries beside ASV | 10/10 |
| SoftwareSerial and math | 10 | Software UART, matrix computation, ADC, and Serial | 10/10 |

## Execution method

1. Generate all 120 complete sketches.
2. Compile every sketch for the Uno before beginning physical uploads.
3. Confirm the expected Uno is still present on COM6 before every upload.
4. Upload each case once and stop the matrix on any upload failure.
5. Run the release desktop application against the physical board.
6. Validate the v0.5.1 board hello, connection state, expected telemetry, frontend/backend agreement, diagnostics, CRC count, packet warnings, and user-Serial drops.
7. Force a disconnect/reconnect scenario in every twelfth case.

## Results

- Compilation: **120/120 passed**.
- Physical uploads: **120/120 succeeded**.
- Corrected functional result: **119/120 passed on the initial run**.
- Focused repeat of the one real failed workload: **5/5 passed without reflashing between sessions**.
- Forced reconnect scenarios: **10/10 completed the expected disconnect and reconnect sequence**.
- Application crashes: **0**.
- GPIO updates received: **8,332**.
- ADC samples received: **2,564**.
- PWM updates received: **6,442**.
- Normal user Serial bytes received: **70,527**.
- CRC failures: **0**.
- Dropped-packet warnings: **0**.
- Dropped user-Serial bytes: **0**.
- Compiled flash range: **5,100–6,950 bytes** (15.8–21.5% of Uno program storage).
- Compiled static SRAM range: **319–551 bytes** (15.6–26.9% of Uno SRAM).

## Harness false positives

The raw automated summary reported five EventQueue cases as failures because it required D4 to show both HIGH and LOW observations. In cases C042, C044, C046, C048, and C050, the generated arithmetic was intentionally even and the queued expression `value & 1` therefore always commanded LOW. The app correctly recorded all of those LOW commands, maintained UI/backend agreement, and produced no protocol diagnostics. These are test-expectation errors, not product failures.

The raw result was therefore 114/120, while the corrected product result is 119/120.

## Real incident: C073 MultiPwmEngine

One initial run that rapidly updated all six hardware PWM outputs produced 12 diagnostics:

- three missing-packet reports;
- five out-of-order/duplicate sequence reports;
- two malformed-length reports;
- no CRC failures;
- no dropped-packet warning counter;
- no user-Serial drops;
- no crash;
- GPIO/PWM frontend state still matched the backend.

The exact C073 firmware was uploaded once more and tested through five independent desktop connection sessions. Every focused session remained connected and received approximately 90 GPIO updates, 661–663 PWM updates, and 1,151–1,162 user-Serial bytes. All five recorded zero diagnostics, zero CRC failures, zero packet warnings, zero Serial drops, and zero crashes.

The incident is therefore intermittent and was not reproduced. It must remain a known beta risk because one genuine malformed/sequence burst was observed under sustained six-channel PWM telemetry.

## Release interpretation

The v0.5.1 application and library are compatible with all 12 tested workload families, including Arduino core libraries, normal user Serial traffic, timer consumers, and reconnects. The matrix did not reveal a deterministic incompatibility or crash.

Before calling v0.5.1 fully release-qualified, run an extended high-rate MultiPwmEngine soak and investigate the single transient framing/sequence burst if it recurs. This 120-sketch matrix does not replace the planned long-duration memory/stability soak or electrical validation with external instruments.

## Evidence

Raw artifacts are stored locally under:

`work/hardware-validation/complex-120-20260812-175757/`

That directory contains all generated sketches, compile logs, build outputs, upload logs, 120 application reports, the raw `summary.json`, and the five-session focused C073 repeat.
