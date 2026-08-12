# ADR 0011: Signed GitHub application updates

## Status

Accepted for v0.6 development.

## Context

Installed classroom and personal copies need to discover newer stable ASV
versions without asking users to monitor the repository manually. Preview
builds may also be published as GitHub prereleases, but they must not pull a
classroom installation off the stable channel.

An installer is executable software. Downloading the first release asset that
looks appropriate would let compromised metadata or an incorrect upload run
untrusted code on a user's computer.

## Decision

The Rust desktop layer queries the public GitHub Releases API and selects the
highest published, non-draft, non-prerelease version that includes
`latest.json`. Draft and preview releases remain invisible. A future explicit
preview-channel preference may opt into prereleases without weakening the
stable default.

The selected `latest.json` is handed to Tauri's native updater. Tauri compares
the installable application version, downloads the matching Windows or macOS
artifact, and requires a valid minisign signature from the public key embedded
in the application before installation. Signature verification cannot be
disabled.

React presents typed metadata and the three user choices:

- **Download & install** downloads, verifies, installs, and restarts;
- **Skip this version** persists only that exact version;
- **Not now** dismisses the current prompt without suppressing future checks.

The updater has no dependency on the board connection manager. GitHub, update,
or preference failures therefore cannot change serial ownership or the
validated GPIO, ADC, PWM, and Serial paths.

## Consequences

Every published update must be built with the same private updater key. The
private key is stored outside Git and in the GitHub Actions secret
`TAURI_SIGNING_PRIVATE_KEY`, with its password in
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`; the public key is safe to ship in
`tauri.conf.json`. Losing the private key prevents existing installations from
trusting future releases, so it requires a protected offline backup.

The stable workflow generates signed updater artifacts and a merged
`latest.json` for Windows x64, Apple-silicon macOS, and Intel macOS. Its asset
audit rejects an incomplete draft. A release without a complete updater
manifest is downloadable manually but will not be offered by the app.

Updater signatures establish project update provenance. They do not provide a
commercial Windows Authenticode identity or Apple notarization, so SmartScreen
and Gatekeeper confirmation may still be required.
