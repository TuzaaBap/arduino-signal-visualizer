# v0.5.1 beta release validation

Date: 2026-08-12

Status: Windows x64 and Arduino Uno release candidate passed. Publication is
pending the release commit/tag and macOS CI artifact verification. Commercial
Windows signing and macOS signing/notarization are not part of this beta.

## Candidate identity

- Base commit: `6cc9c4e82b2eed60e883243c19ea91f036439176`.
- Application, firmware, library, and protocol version: `0.5.1`.
- Version consistency: passed across 11 checked sources.
- Board: Arduino Uno on COM6, FQBN `arduino:avr:uno`.
- USB identity: VID `2341`, PID `0043`, serial
  `7583435373035140D122`.

The candidate was tested in an isolated detached worktree so the later v0.6.0
development commit on `main` was not rewritten or mixed into the v5 artifacts.

## Release fixes found during validation

Security checks found and corrected four dependency advisories before the final
artifacts were built:

- `nanoid` 3.3.16 to 3.3.18;
- `time` 0.3.36 to 0.3.47;
- `plist` 1.7.0 to 1.10.0;
- `quick-xml` 0.32.0 to 0.41.0.

After these lockfile-only updates, `npm audit --audit-level=high` and
`cargo audit` both completed with zero vulnerabilities. Cargo Audit still
reports 17 allowed ecosystem warnings, primarily unmaintained GTK3 packages in
Tauri's Linux dependency graph. Those Linux-only packages are not compiled into
the Windows or macOS v5 targets. They remain maintenance debt and must not be
represented as resolved.

## Automated verification

| Gate | Result |
| --- | ---: |
| Rust tests | 49 passed, 0 failed |
| Frontend tests | 21 passed across 7 files, 0 failed |
| Rust formatting | Passed |
| Clippy with warnings denied | Passed |
| TypeScript and Vite production build | Passed |
| Native C++ shared protocol vectors | GPIO, ADC, and PWM passed |
| Arduino source sketches | 20 of 20 compiled with warnings enabled |
| Packaged ZIP examples | 5 of 5 compiled from an isolated install |
| npm vulnerabilities | 0 |
| RustSec vulnerabilities | 0 |

The 20 Arduino builds consist of the five public library examples and 15
beginner/classroom sketches. Flash use was 11-16% of the Uno's 32,256-byte
application area. The largest classroom sketch used 5,312 bytes of flash and
325 bytes of SRAM.

## Physical classroom matrix

All 15 beginner sketches were compiled, uploaded once per suite entry, and
validated through the real Windows application. Coverage included empty
`setup()`/`loop()`, Blink, multiple and all digital pins, input pull-up, Serial,
ADC, threshold logic, PWM fade, all six Uno PWM pins, mixed telemetry, and a
digital sweep.

| Measurement | Result |
| --- | ---: |
| Scenarios | 15 passed, 0 failed |
| GPIO updates | 506 |
| ADC samples | 63 |
| PWM updates | 187 |
| User-Serial bytes | 824 |
| CRC failures | 0 |
| Protocol diagnostics | 0 |
| Dropped-packet warnings | 0 |
| Dropped user-Serial bytes | 0 |

Raw evidence is retained locally under
`work/hardware-validation/school-suite-20260812-031605/`.

## Deliberate overload and recovery

The overload sketch generated GPIO, ADC, PWM, and normal Serial traffic faster
than a classroom sketch while the application performed a controlled
disconnect/reconnect.

| Measurement | Result |
| --- | ---: |
| Test duration | 34.5 s |
| GPIO updates | 808 |
| ADC samples | 4,052 |
| PWM updates | 2,890 |
| User-Serial bytes | 4,536 |
| UI ADC/PWM maximum buffer | 180 / 180 samples |
| CRC, diagnostic, packet-drop, Serial-drop counts | 0 / 0 / 0 / 0 |

The connection history was
`waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.

## Thirty-minute stability gate

The `SafeMixedStream` sketch used ordinary Arduino calls and continuously
generated GPIO, ADC, PWM, and user-Serial traffic. Its firmware used 5,364
bytes of flash (16%) and 362 bytes of SRAM (17%), leaving 1,686 bytes for local
variables.

| Measurement | Result |
| --- | ---: |
| Physical test window | 1,805.842 s |
| Memory samples | 61 |
| GPIO updates | 18,026 |
| ADC samples | 353,859 |
| PWM updates | 90,117 |
| User-Serial bytes | 646,578 |
| Maximum ADC UI buffer | 180 samples |
| Maximum PWM UI buffer | 180 samples |
| CRC failures | 0 |
| Protocol diagnostics | 0 |
| Dropped-packet warnings | 0 |
| Dropped user-Serial bytes | 0 |

The same application process recovered at the midpoint through
`waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.
GPIO, ADC, and PWM UI state all matched the backend after recovery.

Working set started at 26.805 MiB, ended at 27.320 MiB, and remained between
26.793 and 27.328 MiB. Private memory started at 6.762 MiB, ended at 7.668 MiB,
and remained between 6.707 and 7.863 MiB. Both metrics were exactly flat across
the final ten-minute samples, so no unbounded memory growth was observed.

After the dependency security updates, the rebuilt application completed a
second 184.8-second real-Uno regression with forced reconnect, 1,791 GPIO
updates, 35,294 ADC samples, 8,941 PWM updates, 62,532 user-Serial bytes, all UI
matches true, and all integrity counters zero.

Raw stability evidence is retained locally under
`work/hardware-validation/v5-stability-20260812-032159/` and the post-security
regression under
`work/hardware-validation/v5-security-regression-20260812-040233/`.

## Full-I/O hardware confirmation

A second 30-minute physical run exercised the complete v5 classroom workload
at once. Six ordinary digital outputs (D2, D4, D7, D8, D12, and D13) toggled,
all six hardware-PWM outputs faded from 0 through 255, all six analog inputs
were sampled, and normal user Serial text shared the UART with ASV telemetry.
D3, D5, D6, and D9 were connected directly to A1, A3, A4, and A5 to verify
the PWM LOW/HIGH endpoints. A0 and A2 remained connected to the external
10 Hz waveform generator. The test firmware used 6,054 bytes of flash (18%)
and 406 bytes of SRAM (19%).

| Measurement | Result |
| --- | ---: |
| Physical test window | 1,805.828 s |
| Memory samples | 61 |
| GPIO updates | 43,063 |
| ADC samples | 400,393 |
| PWM updates | 93,162 |
| User-Serial bytes | 65,495 |
| Maximum ADC UI buffer | 180 samples |
| Maximum PWM UI buffer | 180 samples |
| CRC failures | 0 |
| Protocol diagnostics | 0 |
| Dropped-packet warnings | 0 |
| Dropped user-Serial bytes | 0 |

Every PWM pin produced exactly 15,527 updates and covered duty values 0-255.
The direct loopback channels A1, A3, A4, and A5 each covered raw ADC values
0-1023. A0 and A2 each produced 66,732 samples and covered 0-573 counts from
the external generator. GPIO, ADC, and PWM UI state all matched the backend.

The same application process completed the midpoint recovery sequence
`waitingForHello -> connected -> disconnected -> waitingForHello -> connected`
without a diagnostic or dropped packet. Across steady-state samples after the
first ten minutes, working-set slope was +0.15 KiB/min, private-memory slope was
-2.76 KiB/min, and the handle-count range was two. No unbounded growth was
observed.

An immediately preceding 30-minute run observed one two-packet sequence gap
at its forced reconnect boundary. CRC, queue-drop, Serial-drop, UI-state, and
memory checks remained clean. Five consecutive short reconnect stress cycles
then completed with zero diagnostics, followed by the clean full confirmation
above. The sequence detector was intentionally retained: suppressing a
non-reproducible warning would also hide genuine within-session packet loss.

Raw evidence is retained locally under
`work/hardware-validation/full-system-stability-20260812-050503/`,
`work/hardware-validation/reconnect-stress-20260812-053828/`, and
`work/hardware-validation/full-system-confirmation-20260812-054053/`.

## Release artifacts

| Artifact | Size | SHA-256 |
| --- | ---: | --- |
| `ArduinoSignalVisualizer-0.5.1.zip` | 37,972 B | `DF5B3144E39973E1894196692211504C29A014B9A21C586F14A08A6DA3FF2D74` |
| `arduino-signal-visualizer.exe` | 9,108,480 B | `D29B52DC53F77F6B5C844969BBA8CB0260A4130147C029203991BBB685C6402A` |
| `Arduino Signal Visualizer_0.5.1_x64_en-US.msi` | 3,215,360 B | `9D3F80B6AEA629936ECAE2F91DA1829A1F468DFD28F77D7ED62DA4BD52F68B33` |
| `Arduino Signal Visualizer_0.5.1_x64-setup.exe` | 2,173,329 B | `725CDED46C062A96F8A011342BCD9CD73EDBA751E30508B9C345BE002B31D631` |

The Arduino ZIP was generated twice with the same hash, installed into an
isolated Arduino user directory, and all five installed examples compiled. The
NSIS installer completed a silent current-user installation, registered v0.5.1
once in Windows Start search, and the freshly installed application passed its
startup smoke test. The MSI completed an administrative extraction through
Windows Installer.

## Known beta boundaries

- Windows artifacts are not Authenticode-signed. SmartScreen may require user
  confirmation.
- macOS artifacts must be rebuilt and checked by the macOS CI runner before
  publication. They are not commercially signed or notarized, so Gatekeeper
  confirmation may be required.
- D0/D1 remain the Uno's physically shared USB UART pins.
- Transparent Serial mode supports ordinary text. Arbitrary unframed binary
  traffic containing ASV delimiters requires a future strict-binary mode.
- Instrumentation reports configured MCU state. It is not an electrical logic
  analyzer and cannot observe edges generated outside instrumented calls.
- The Uno has finite UART bandwidth. Under overload ASV retains the newest
  state per signal rather than claiming every physical edge was captured.

## Release decision

The Windows x64 plus Arduino Uno v0.5.1 beta candidate passes the functional,
protocol, packaging, installer, reconnect, bounded-buffer, and 30-minute
stability gates. Do not publish the release until the lockfile fixes and this
report are committed, macOS CI succeeds, artifact hashes are captured, and the
unsigned-beta limitation is displayed prominently in the release notes.
