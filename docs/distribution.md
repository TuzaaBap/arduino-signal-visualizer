# Distribution and beta release process

## Release contents

One beta tag produces matching artifacts for the same source revision:

| Audience | Artifact | Build host |
| --- | --- | --- |
| Windows 10/11 x64 | NSIS `-setup.exe` (recommended, current user) | `windows-latest` |
| Managed Windows x64 | MSI (administrator, all users) | `windows-latest` |
| Apple-silicon Mac | `aarch64` DMG | `macos-15` |
| Intel Mac | `x86_64` DMG | `macos-15-intel` |
| Arduino IDE 2 | `ArduinoSignalVisualizer-VERSION.zip` and SHA-256 | `ubuntu-latest` |

Desktop bundles are built by Tauri on the native operating system. The Arduino
ZIP is deterministic and has one top-level `ArduinoSignalVisualizer` folder,
matching the official Arduino library layout. Its install test uses a fresh
Arduino user directory and compiles all five installed examples.

## Manual beta workflow

The **Beta Release** GitHub Actions workflow requires an explicit tag input such
as `v0.5.0-beta.1`. The tag must match the version in `library.properties`.
The workflow creates a draft prerelease, then attaches all successful platform
artifacts. A maintainer reviews the draft and download names before making it
public.

Before starting the workflow:

1. Confirm `main` CI passes on Windows, macOS, and Arduino Uno jobs.
2. Confirm the version agrees in Cargo, npm, Tauri, firmware hello, and
   `library.properties`.
3. Complete the physical validation required by the feature milestone.
4. Review the beta notes and known limitations.

## Local Arduino ZIP

Create the same deterministic package used by CI:

```powershell
python scripts/package_arduino_library.py --output-dir outputs/release
```

The script fails if metadata, source files, or any validated example is missing.
It writes the ZIP and a sibling `.sha256` checksum file.

## Serial ownership and compatibility

The library preserves the Arduino core `Serial` object. ASV protocol-v2 frames
and ordinary sketch text share the Uno UART but are decoded into separate
desktop streams. User text has priority; ASV skips telemetry rather than
blocking the sketch if the Uno transmit buffer is full.

Including `ASVInstrumented.h` enables the automatic lifecycle. It starts Serial
at 115200 baud by default, then attaches telemetry after the sketch's `setup()`.
If the sketch calls `Serial.begin(baud)`, that user-selected rate wins. The
advanced explicit API remains available through `ArduinoSignalVisualizer.h`.
The desktop connection panel must use the sketch's baud because one physical
UART cannot operate at two baud rates simultaneously.

The operating-system serial port is exclusive. Arduino IDE Serial Monitor,
another terminal, and Arduino Signal Visualizer cannot connect simultaneously.
The application opens the port only after the user selects **Connect**, releases
it on **Disconnect** or application exit, and provides its own Serial tab for
normal sketch input/output.

Transparent mode supports ordinary text. Strict framing of arbitrary binary
user traffic is outside the 0.5.0 beta and must not be implied in release copy.

## Signing status

The initial beta is intentionally labelled unsigned:

- Windows installers run but may trigger Microsoft SmartScreen because no
  public code-signing certificate is configured.
- macOS bundles use Tauri's ad-hoc identity (`-`). Gatekeeper still requires the
  user to allow the application in **Privacy & Security**.

Public production distribution requires credentials, not a source-code
workaround. The production gate is:

1. Windows code-signing identity or managed signing service.
2. Apple Developer ID Application certificate.
3. Apple notarization credentials and successful stapling.
4. Clean-machine installation and uninstall tests on both operating systems.
5. Published SHA-256 checksums and final release notes.

References:

- [Tauri distribution](https://v2.tauri.app/distribute/)
- [Tauri Windows installers](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/)
- [Arduino library specification](https://docs.arduino.cc/arduino-cli/library-specification)
