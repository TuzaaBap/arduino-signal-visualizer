# Distribution and release process

## Release contents

One release tag produces matching artifacts for the same source revision:

| Audience            | Artifact                                          | Build host       |
| ------------------- | ------------------------------------------------- | ---------------- |
| Windows 10/11 x64   | NSIS `-setup.exe` (recommended, current user)     | `windows-latest` |
| Managed Windows x64 | MSI (administrator, all users)                    | `windows-latest` |
| Apple-silicon Mac   | `aarch64` DMG                                     | `macos-15`       |
| Intel Mac           | `x86_64` DMG                                      | `macos-15-intel` |
| Arduino IDE 2       | `ArduinoSignalVisualizer-VERSION.zip` and SHA-256 | `ubuntu-latest`  |

Desktop bundles are built by Tauri on the native operating system. The Arduino
ZIP is deterministic and has one top-level `ArduinoSignalVisualizer` folder,
matching the official Arduino library layout. Its install test uses a fresh
Arduino user directory and compiles all five installed examples.

Each desktop build also creates a signed updater artifact. The release action
merges Windows, Apple-silicon macOS, and Intel macOS entries into one
`latest.json` asset. Published application versions can then discover the
release and offer **Download & install**, **Skip this version**, or **Not now**.

## Stable workflow

The **Stable Release** GitHub Actions workflow derives the exact `vX.Y.Z` tag
from the version checked into `main`. It creates a draft, builds all desktop and
Arduino artifacts, and runs `scripts/audit_release_assets.py`. The audit rejects
the draft if an installer, updater platform, signature, Arduino ZIP, or download
target is missing. It also publishes `SHA256SUMS.txt`.

The workflow never publishes automatically. After all jobs pass, a maintainer
must inspect the draft and explicitly make it public. This keeps a partial
Windows/macOS matrix from ever becoming the public stable release.

## Preview workflow

The **Beta Release** GitHub Actions workflow remains available for an explicit
preview tag input such
as `v0.5.1-beta.1`. The tag must match the version in `library.properties`.
The workflow creates a draft prerelease, then attaches all successful platform
artifacts. A maintainer reviews the draft and download names before making it
public.

Before starting the workflow:

1. Confirm `main` CI passes on Windows, macOS, and Arduino Uno jobs.
2. Confirm the version agrees in Cargo, npm, Tauri, firmware hello, and
   `library.properties`.
3. Complete the physical validation required by the feature milestone.
4. Review the release notes and known limitations.
5. Confirm the repository Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` exist.

Every installable release must increment the application SemVer. Tags such as
`v0.7.0-beta.1` and `v0.7.0-beta.2` both install application version `0.7.0`,
so the second tag is a replacement build rather than an update for users who
already installed the first. Use `0.7.1` (or the next intended version) when an
existing installed beta must receive another updater notification.

After every platform job succeeds, inspect the draft release before publishing:

1. Confirm `latest.json` exists.
2. Confirm it contains `windows-x86_64`, `darwin-aarch64`, and
   `darwin-x86_64` platform entries with non-empty signatures and URLs.
3. Confirm the matching updater bundles and `.sig` assets exist.
4. Test an update from the preceding installed version on Windows and macOS.

The updater private key is not a replaceable build cache. It must remain outside
Git, be backed up in a protected offline location, and match the public key in
`desktop/src-tauri/tauri.conf.json`. Losing it means existing installations
cannot authenticate later releases.

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
desktop streams. ASV keeps bounded latest-state telemetry slots and sends only
complete frames that fit the Uno transmit buffer. A busy UART therefore neither
blocks the sketch nor creates partial frames or false sequence gaps.

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
user traffic is outside v0.6 and must not be implied in release copy.

## Signing status

The v0.6 stable release has cryptographically signed updater packages, but its
operating-system installers do not yet have commercial publisher identities:

- Windows installers run but may trigger Microsoft SmartScreen because no
  public code-signing certificate is configured.
- macOS bundles use Tauri's ad-hoc identity (`-`). Gatekeeper still requires the
  user to allow the application in **Privacy & Security**.

Application updates are separately signed with Tauri's updater key so installed
ASV copies can reject altered update packages. That technical package signature
does not identify the publisher to Windows or Apple and does not remove
SmartScreen or Gatekeeper confirmation.

Removing the Windows and macOS trust prompts requires credentials, not a
source-code workaround. The commercial-signing gate is:

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
