# Architecture

## Data flow

```text
Instrumented Uno sketch
  -> Arduino ASV encoder
  -> COBS/CRC binary frames
  -> Rust serial reader
  -> framing and typed decoder
  -> bounded delivery queue
  -> recording/validation branch (all validated ADC events)
  -> 30 Hz GPIO and ADC UI batches
  -> bounded React state and interactive Uno SVG
```

The Arduino side reports intent and observed return values. The Rust side treats
every serial byte as untrusted and is the only layer that understands packet
layout. React only understands typed concepts such as pin, direction, logic
level, raw ADC count, resolution, and integer reference metadata. It never
parses wire bytes.

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
retains the newest value. React then keeps the latest 180 samples per channel
for the small trend graphs. These separate bounds prevent both native and UI
memory growth without changing the validated GPIO path.

## Mock Mode

Mock Mode is clearly labelled and produces deterministic D2-D13 GPIO and A0-A5
ADC events through the same typed delivery path as serial data. It does not
simulate electrical behavior and is not presented as a connected board.
