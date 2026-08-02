# Development setup

## Supported hosts

- Windows 11
- macOS 14 or newer

## Required dependencies

- Git
- Node.js 24 LTS and npm 11 or newer
- Rust 1.97.1 through rustup, including `rustfmt` and `clippy`
- Arduino CLI 1.5 or newer
- Arduino AVR board core
- Windows: WebView2 Runtime and Visual Studio 2022 C++ Build Tools with a
  Windows SDK
- macOS: Xcode Command Line Tools

The exact Rust release is pinned by `rust-toolchain.toml`. JavaScript
dependencies are pinned by `package-lock.json`.

## Windows installation

The following package IDs can be installed with WinGet:

```powershell
winget install --id OpenJS.NodeJS.LTS --exact --scope user
winget install --id Rustlang.Rustup --exact
winget install --id ArduinoSA.CLI --exact
winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Open a new terminal after installation so `PATH` is refreshed.

## macOS installation

Install Xcode Command Line Tools and use the official Node.js, rustup, and
Arduino CLI distributions. Tauri uses the system WebKit runtime.

## Repository setup

```powershell
npm.cmd install
rustup component add rustfmt clippy
arduino-cli core update-index
arduino-cli core install arduino:avr
```

## Verify the environment

Windows:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass `
  -File ./scripts/verify-environment.ps1
```

macOS:

```bash
./scripts/verify-environment.sh
```

## Run

Browser-only UI development:

```powershell
npm.cmd run dev
```

Desktop application:

```powershell
npm.cmd run tauri -- dev
```

Production build:

```powershell
npm.cmd run tauri -- build
```

The equivalent compile-only path, useful in locked-down development
environments, is:

```powershell
npm.cmd run build
cargo build --release -p asv-desktop --features custom-protocol
```

`custom-protocol` embeds the production frontend in the executable. Omit it
for a debug build that intentionally connects to the Vite development server.

Windows Smart App Control can block unsigned Rust build scripts and PlatformIO
tools. It has no per-application exception. Prefer an approved signed toolchain,
CI runner, or development machine whose security policy is intended for local
compilation. Turning Smart App Control off is a global security decision, not a
project setup step.

## Hardware

1. Install the Arduino AVR core.
2. Compile and upload either `firmware/examples/GpioDemo/GpioDemo.ino` or the
   explicit Milestone 2 target `firmware/examples/AdcDemo/AdcDemo.ino`.
3. Close Arduino Serial Monitor so the desktop app can own the serial port.
4. Select the port in the app and choose **Connect**.

The example uses 115200 baud. The Uno normally resets when the serial port is
opened; the desktop waits for the ASV hello frame.
