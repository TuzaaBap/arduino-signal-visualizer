# Milestone 2 validation: ADC visualisation

Validated on Windows 11 with an Arduino Uno R3 on 2026-08-02.

## Hardware and firmware

- Board: Arduino Uno R3, USB VID/PID `2341:0043`, serial
  `7583435373035140D122`.
- Port: `COM6` at 115200 baud.
- Firmware: `firmware/ArduinoSignalVisualizer/examples/AdcDemo/AdcDemo.ino`,
  version `0.2.0`.
- PlatformIO target: `uno_adc_demo`, `atmelavr@5.3.0`, Arduino framework.
- Uploader: avrdude 6.3 using the `arduino` protocol at 115200 baud.
- Upload result: all 3,006 bytes written and verified on the first attempt.
- Flash: 3,006 of 32,256 bytes (9.3%).
- SRAM: 230 of 2,048 bytes (11.2%).
- Uploaded HEX SHA-256:
  `B19CC31ECBC4A46DE1620BFA6DEA7C07E6C77970DE1A30B4280E805A1AE7C277`.
- GPIO regression firmware still builds independently at 3,138 bytes flash
  and 236 bytes SRAM.

## Physical ADC results

The tests used A0, 10-bit resolution, default reference mode, and declared
reference metadata of 5,000 mV. No multimeter readings were supplied.

| Test point | Expected raw | Observed raw | Calculated voltage | Multimeter | Absolute error | Percentage error |
| --- | ---: | ---: | ---: | --- | ---: | ---: |
| GND | 0-5 | 0-0 | 0.000 V | Not supplied | 0 mV | N/A at 0 V |
| Equal-resistor midpoint | 486-537 | 511-514 | 2.508 V average | Not supplied | 7.8 mV from ideal 2.500 V | 0.31% |
| Uno 5 V/reference rail | 1018-1023 | 1023-1023 | 5.000 V | Not supplied | 0 mV from declared 5.000 V | 0.00% |

Each point used 11 unique report samples over approximately 10 seconds. The
midpoint expected range allows nominally equal resistors with up to 5%
tolerance. Calculated error is relative to the declared reference metadata; it
does not replace an independent multimeter measurement.

## Production stability validation

The authoritative run used the optimized release executable with its frontend
embedded through Tauri's `custom-protocol` feature. The Analog tab and all six
live graphs remained mounted during the run.

- Observation duration: 1,800.899 seconds.
- Memory samples: 61 at approximately 30-second intervals.
- ADC events preserved by the validation/recording branch: 220,818.
- Per-channel ADC events: 36,803 on each of A0-A5.
- GPIO updates received concurrently: 3,835.
- ADC UI acknowledgements: 46,467.
- GPIO UI acknowledgements: 3,834.
- Maximum React history: 180 samples per channel, exactly the configured bound.
- Backend/UI ADC agreement was observed.
- Controlled disconnect/reconnect history:
  `waitingForHello -> connected -> disconnected -> waitingForHello -> connected`.
- Protocol diagnostics: 0.
- CRC failures: 0.
- Invalid packets: 0.
- Dropped-packet or queue-pressure warnings: 0.
- Application crashes: 0.

| Memory metric | Start | End | Minimum | Maximum | Final 10-minute range | Final 10-minute slope |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Working set | 30,466,048 B | 30,842,880 B | 30,396,416 B | 30,863,360 B | 24,576 B | 1.42 KiB/min |
| Private bytes | 6,496,256 B | 7,364,608 B | 5,910,528 B | 7,528,448 B | 32,768 B | -0.91 KiB/min |

The initial increase occurred while the Analog view and WebView resources were
warming up. Both metrics then reached a narrow plateau; private memory was
slightly decreasing over the final ten minutes. No unbounded memory growth was
observed.

## Protocol and architecture changes

- Added packet type `0x11`, ADC event schema version 1.
- Payload fields are schema version, channel, raw count, resolution bits,
  reference mode, and integer reference millivolts. Sequence and board time
  remain in the common header.
- The Uno transmits no floating-point voltage values.
- Rust rejects malformed length, unsupported event version or resolution,
  invalid channel/reference metadata, and raw counts outside the declared
  resolution.
- Every valid ADC event reaches the recording/validation branch before UI
  coalescing. The Rust UI queue is bounded per channel and always retains the
  latest value; React retains 180 graph samples per channel.
- The validated GPIO path remains independent and was exercised throughout the
  ADC run by D13 updates.
- Rust and C++ consume the same GPIO and ADC binary vectors.

Raw ADC counts are canonical because they are the actual converter result.
Voltage is derived later so improved reference calibration can be applied
without losing or rewriting the original measurement.

## Software validation

- 17 Rust protocol tests passed, including ADC length, version, resolution,
  channel, raw-range, CRC, and sequence cases.
- 2 Rust connection/backend tests passed.
- 6 frontend state tests passed, including bounded ADC history and voltage
  calculations.
- Native C++ shared-vector validation passed for GPIO and ADC frames.
- Rust formatting, Clippy with warnings denied, TypeScript checking, and the
  frontend production bundle passed.
- Arduino CLI and PlatformIO builds passed for both `GpioDemo` and `AdcDemo`.
- The environment verifier passed for Git, Node.js, npm, Rust, Cargo, Arduino
  CLI, WebView2, and MSVC.
- The production Analog UI was visually verified with live hardware data.

This is 25 automated Rust/frontend tests plus the native cross-language vector
executable and two firmware compile gates.

## Known limitations

- The default 5,000 mV reference is declared nominal metadata, not a measured
  AREF or USB-supply voltage. Accurate voltage work requires a measured and
  explicitly configured reference value.
- The small graphs show trends only. Sampling cadence, USB transport, UI
  batching, and the Uno ADC do not provide oscilloscope-grade timing,
  bandwidth, or calibration.
- The demo samples each channel at 20 samples per second. It is not a maximum
  throughput benchmark.
- Recording UI remains outside Milestone 2. The typed lossless branch exists,
  but a user-facing recording workflow is a later milestone.
- Windows Smart App Control can block unsigned local development binaries and
  has no per-application exception. Production distribution will require an
  approved signing workflow, which remains outside this milestone.

## Proposed release

Proposed tag: `v0.2.0-alpha`. The tag is not created as part of this report.
