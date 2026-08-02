# Proposed Milestone 3 plan: PWM

This is a design proposal only. PWM implementation requires explicit approval.

## Recommended architecture

Add `ASV.analogWrite(pin, value)` as another thin Arduino wrapper: call the real
Arduino function first, preserve its normal behavior, then report the requested
output state. Keep PWM events independent from GPIO and ADC events so adding PWM
cannot destabilize either validated path.

The canonical value should be the integer duty count and declared resolution,
not a floating-point percentage. Desktop code can calculate percentage from
those fields. Timer and carrier-frequency metadata must be explicit integer
units and must be labelled nominal unless independently measured.

## Proposed tasks

1. **Contract and hardware semantics**
   - Decide whether non-PWM Uno pins are rejected or explicitly represented as
     Arduino's digital fallback behavior.
   - Define versioned `PwmWrite` fields: pin, duty count, resolution bits,
     output mode, nominal carrier frequency in integer hertz, sequence, and
     board timestamp.
   - Add malformed, unsupported-pin, duty-range, CRC, sequence, and shared
     Rust/C++ vector tests.

2. **Firmware**
   - Add `ASV.analogWrite(pin, value)` without changing direct Arduino return or
     timing behavior.
   - Cover Uno PWM pins D3, D5, D6, D9, D10, and D11 and their timer groups.
   - Keep the GPIO and ADC examples and regression builds unchanged.

3. **Rust backend**
   - Decode PWM packets into typed board-independent events.
   - Validate pin capability, duty range, resolution, and frequency metadata.
   - Preserve all valid events before bounded UI coalescing.

4. **Frontend**
   - Add a PWM tab and make PWM-capable Uno pins selectable.
   - Show raw duty, calculated percentage, nominal frequency, and a bounded
     state history.
   - Clearly distinguish requested duty from electrically measured waveform
     behavior.

5. **Hardware validation**
   - Test 0%, approximately 25%, 50%, 75%, and 100% duty on representatives of
     each Uno timer group.
   - Compare against a logic analyzer or oscilloscope only when a manual
     measurement is supplied.
   - Repeat disconnect/reconnect, protocol-fault, GPIO/ADC regression, and
     30-minute production stability gates.

PWM work must not expand into UART, SPI, I2C, recording UI, packaging, or
signing unless separately approved.
