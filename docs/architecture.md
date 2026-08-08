# Architecture

## Data flow

```text
Instrumented Uno sketch + ordinary Serial.print output
  -> Arduino ASV v2 encoder + untouched user bytes on one UART
  -> zero-delimited COBS/CRC ASV frames mixed with user text
  -> Rust serial reader and transport demultiplexer
  -> typed ASV decoder + bounded user Serial stream
  -> bounded delivery queue
  -> recording/validation branch (all validated ADC and PWM events)
  -> 30 Hz GPIO, ADC, and PWM UI batches
  -> bounded React state, Serial Monitor, and interactive Uno SVG
```

The Arduino side reports intent, observed return values, and PWM timer-register
snapshots. The Rust side treats every serial byte as untrusted and is the only
layer that understands packet layout or derives PWM timing. React only
understands typed concepts such as pin, direction, logic level, raw ADC count,
reference metadata, and validated timer timing. It never parses wire bytes or
derives a waveform from rounded pin-frequency labels.

## Repository boundaries

- `firmware/ArduinoSignalVisualizer`: small AVR-compatible instrumentation
  library with no dynamic allocation in the encoder.
- `crates/asv-protocol`: reusable Rust framing, validation, sequence tracking,
  and board-independent events.
- `desktop/src-tauri`: serial lifecycle, bounded queues, deterministic mock
  source, and rate-limited Tauri delivery.
- `desktop/src`: React views, application state, and the interactive SVG board.
- `protocol`: canonical wire specification and vectors shared by C++ and Rust.

## Connection lifecycle

Opening an Uno's serial port normally resets the board. The desktop therefore
enters `waitingForHello` after opening the port and only reports `connected`
after a valid ASV board-hello packet. A three-second timeout diagnoses wrong
firmware or baud rate without crashing or guessing.

The byte reader may attach partway through a frame or receive traffic from the
previous sketch instance before the reset completes. It silently discards only
that initial partial fragment and does not accept GPIO events until a valid
board-hello packet establishes the session. Once connected, every framing,
CRC, and sequence error is reported normally.

Disconnect sets one shared stop flag. The reader has a short serial timeout, so
all worker threads can finish promptly before another connection starts.

## Queue behavior

The source-to-delivery queue is bounded to 256 messages. GPIO keeps its existing
non-blocking behavior. Valid ADC samples use backpressure rather than being
dropped, and every sample reaches the recording/validation branch before UI
coalescing.

The delivery worker keeps only the latest GPIO state per pin during each 33 ms
window. ADC UI delivery uses a bounded 64-sample queue per channel and always
retains the newest value. PWM reaches the validation branch before a six-entry
latest-per-pin UI map coalesces delivery. React keeps the latest 180 ADC samples
per channel and 180 PWM timer snapshots per pin. These separate bounds prevent
both native and UI memory growth without changing the validated GPIO or ADC
paths.

## Mock Mode

Mock Mode is clearly labelled and produces deterministic D2-D13 GPIO, A0-A5
ADC, and D3/D5/D6/D9/D10/D11 PWM events through the same typed delivery path as
serial data. It does not simulate electrical behavior and is not presented as a
connected board.

## PWM truth model

PWM has two distinct sources of truth. The current product implements the
configured source: firmware snapshots TCCR, OCR, TCNT, timer/channel, clock,
prescaler, and TOP after `analogWrite()`, and Rust reconstructs the rectangular
timer output using integer arithmetic. React labels this `Configured MCU
waveform` and provides a selectable time window.

An electrically measured source would require capture hardware. The configured
trace cannot reveal voltage levels, loading, noise, rise/fall time, oscillator
error, or wiring faults. Keeping configured and measured sources distinct
prevents the educational UI from making oscilloscope or logic-analyzer claims
that the hardware cannot support.

## Uno R3 board model

The React board view separates physical artwork from electrical meaning. The
SVG provides a physically representative R3 outline, connectors, major
components and headers. `desktop/src/domain/uno-r3-pinout.ts` provides the typed
pin map used by selection behavior, labels and accessibility text.

This is similar to separating a PCB footprint from its netlist: the drawing can
be refined without changing pin behavior, and UART, SPI, I2C or interrupt
milestones can activate an existing capability without creating another board
illustration. Roles for future protocols are visible for teaching, but visibility
does not imply that the corresponding ASV protocol tool is implemented.
