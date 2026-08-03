# Milestone 3 plan: timer-derived PWM visualization

This revision was approved on 2026-08-03. It replaces the original rounded
frequency and requested-duty-history design.

## Recommended architecture

Keep `ASV.analogWrite(pin, value)` as a thin Arduino wrapper: validate the real
Uno PWM pin and count, call Arduino, then snapshot the driving timer registers.
Keep PWM events independent from GPIO and ADC events so the revision cannot
destabilize either validated path.

The canonical values are integer timer configuration and requested duty, not a
floating-point percentage or rounded nominal frequency. Rust calculates exact
configured timing for the reported register state. The UI must label it as
configured rather than electrically measured.

## Proposed tasks

1. **Contract and hardware semantics**
   - Decide whether non-PWM Uno pins are rejected or explicitly represented as
     Arduino's digital fallback behavior.
   - Define versioned `PwmWrite` fields: pin, duty count, resolution, output
     mode, timer/channel, waveform mode, polarity, source clock, prescaler,
     TOP, compare, counter, raw control registers, sequence, and timestamp.
   - Add malformed, unsupported-pin, duty-range, CRC, sequence, and shared
     Rust/C++ vector tests.

2. **Firmware**
   - Add `ASV.analogWrite(pin, value)` without changing direct Arduino return or
     timing behavior.
   - Cover Uno PWM pins D3, D5, D6, D9, D10, and D11 and their timer groups.
   - Keep the GPIO and ADC examples and regression builds unchanged.

3. **Rust backend**
   - Decode PWM packets into typed board-independent events.
   - Validate pin capability, duty range, resolution, timer/channel mapping,
     waveform mode, polarity, clock, prescaler, TOP, compare, and counter.
   - Derive period, frequency, HIGH time, LOW time, and duty with integer units.
   - Preserve all valid events before bounded UI coalescing.

4. **Frontend**
   - Add a PWM tab and make PWM-capable Uno pins selectable.
   - Show a rectangular configured waveform, exact timer-derived timing,
     timer counter and compare values, and selectable time window.
   - Never connect duty-history points into a false analog-looking waveform.
   - Clearly distinguish configured timing from electrically measured behavior.

5. **Hardware validation**
   - Test 0%, approximately 25%, 50%, 75%, and 100% duty on representatives of
     each Uno timer group.
   - Compare against a logic analyzer or oscilloscope only when a manual
     measurement is supplied.
   - Repeat disconnect/reconnect, protocol-fault, GPIO/ADC regression, and
     30-minute production stability gates.

PWM work must not expand into UART, SPI, I2C, recording UI, packaging, or
signing unless separately approved.
