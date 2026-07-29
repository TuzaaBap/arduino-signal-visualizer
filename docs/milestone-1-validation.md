# Milestone 1 validation

Validated on Windows 11 with an Arduino Uno R3 on 2026-07-29.

## Hardware and upload

- Board identity: Arduino Uno R3, USB VID/PID `2341:0043`, serial
  `7583435373035140D122`.
- Port: `COM6`.
- PlatformIO environment: `atmelavr@5.3.0`, board `uno`, Arduino framework.
- Uploader: avrdude 6.3, `arduino` protocol, 115200 baud.
- Firmware: `firmware/examples/GpioDemo/GpioDemo.ino`.
- Flash use: 3,122 of 32,256 bytes (9.7%).
- SRAM use: 233 of 2,048 bytes (11.4%).
- Uploaded HEX SHA-256:
  `7E4FFA49499C7C4DCD25BF25513BABA7080A1B35569123C9AB2F8DE884277E3B`.
- Avrdude wrote and verified all 3,122 bytes.

## GPIO validation checklist

- [x] Desktop detected the Arduino Uno R3 on COM6.
- [x] Firmware version `0.1.0` was reported correctly.
- [x] The serial connection remained stable.
- [x] D13 toggled correctly: 4,095 updates, including 2,047 HIGH and 2,048
  LOW observations.
- [x] D2-D12 all updated repeatedly and each produced both HIGH and LOW
  observations.
- [x] UI state matched the backend GPIO direction and level during sampled
  synchronization points.
- [x] Controlled disconnect was detected.
- [x] Automatic reconnect returned through `waitingForHello` to `connected`.
- [x] No protocol diagnostics were recorded in the final run.
- [x] No CRC failures were recorded.
- [x] No dropped-packet or queue-pressure warnings were recorded.
- [x] The application remained responsive and did not crash during the run.
- [x] No unbounded memory growth was observed during the 30-minute run.

The final hardware session processed 36,966 GPIO updates and 20,594 UI
acknowledgements. The connection history was:

```text
waitingForHello -> connected -> disconnected -> waitingForHello -> connected
```

## Memory observation

The dedicated observation ran for 1,800.95 seconds with 61 samples at 30-second
intervals.

| Metric | Start | End | Minimum | Maximum |
| --- | ---: | ---: | ---: | ---: |
| Desktop working set | 31,346,688 B | 33,484,800 B | 31,313,920 B | 33,501,184 B |
| Desktop private bytes | 5,820,416 B | 5,992,448 B | 5,758,976 B | 6,111,232 B |

Private memory stayed within a 345 KiB band and ended below its warm-up peak.
Its fitted slope was approximately 5.7 KiB/minute, with long flat and downward
periods rather than monotonic growth. No runtime memory leak was observed in
this 30-minute test.

Raw validation artifacts are generated under `work/hardware-validation/` and
are intentionally excluded from version control.

## Software validation

- Environment verifier passed for Git, Node.js, npm, Rust, Cargo, Arduino CLI,
  WebView2, and MSVC.
- Frontend strict TypeScript check passed.
- Three frontend GPIO state tests passed.
- Frontend production bundle passed.
- Nine Rust protocol tests passed, including live-stream resynchronization.
- Deterministic Mock Mode test passed.
- Rust workspace debug build passed.
- Arduino CLI and PlatformIO Uno builds produced the same flash and SRAM use.
- Native MSVC execution passed the shared Rust/C++ protocol vector.
- Visual checks passed at 1280x800 and the 960x640 minimum window size.
- Keyboard SVG-pin selection and browser console checks passed.

## Remaining platform checks

- The macOS build remains configured in `.github/workflows/ci.yml` and requires
  a macOS runner; Windows cannot link Tauri's Objective-C/WebKit dependencies.
- This host's Windows Application Control policy can block newly generated
  release helpers and Tauri's prebuilt npm CLI binding. Debug compilation and
  the hardware validation executable run successfully. Release packaging must
  run in CI or an administrator-approved signing/allow-list environment.
