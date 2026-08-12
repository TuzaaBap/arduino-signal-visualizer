# Arduino firmware library

The product-facing header lets sketches keep ordinary Arduino function names
while ASV reports the operations to the desktop. Include third-party headers
first and `ASVInstrumented.h` last:

## Instrumentation API

```cpp
#include <ASVInstrumented.h>

void setup() {
  pinMode(13, OUTPUT);
  Serial.println("Normal user Serial output");
}

void loop() {
  digitalWrite(13, HIGH);
  delay(1000);
  digitalWrite(13, LOW);
  delay(1000);
}
```

The include automatically starts ASV at 115200 baud, attaches after the
user's `setup()` completes, and services pending telemetry around each normal
`loop()` call. The sketch does not call `ASV.begin()`, `ASV.attach()`, or
`ASV.service()`.

The header redirects sketch calls to `pinMode`, `digitalWrite`, `digitalRead`,
`analogReference`, `analogRead`, and `analogWrite`. The underlying Arduino core
and separately compiled third-party libraries are unchanged. The explicit
`ArduinoSignalVisualizer.h` header remains available when `ASV.digitalWrite()`
style calls are preferred.

If the sketch calls normal `Serial.begin(baud)` in `setup()`, that speed wins;
select the same baud in the desktop application. If it does not, ASV uses
115200 baud. The explicit `ArduinoSignalVisualizer.h` API retains `ASV.begin`
and `ASV.attach` for advanced integrations that do not include
`ASVInstrumented.h`.

## Shared UART and normal Serial output

Protocol v2 puts ASV telemetry inside signed, COBS-encoded, CRC-protected frames
with explicit start and end delimiters. Normal `Serial.print()` bytes remain
untouched. The desktop separates the two streams and shows only the user's raw
bytes in the Serial Monitor tab.

Desktop Serial Monitor input is written as raw bytes, so ordinary
`Serial.available()` and `Serial.read()` continue to work. ASV control commands
do not share the desktop-to-board direction in this version.

ASV telemetry never waits behind a full Uno transmit buffer. GPIO, ADC, and PWM
use bounded latest-state slots and are serviced fairly when a complete frame
fits. A busy UART does not create a partial ASV frame or a false sequence gap;
if calls arrive faster than the physical link can carry them, the newest pending
state replaces the older pending state for that signal.

Telemetry is paced to at most two thirds of the configured UART's 8N1 wire
capacity. The remaining third is reserved for normal sketch Serial output and
USB bridge timing margin. This rate limit does not add a growing queue or delay
the user's loop; it operates on the same bounded latest-state slots.

Instrumented `analogRead()` returns the Arduino core result unchanged, then reports the
channel, raw count, resolution, reference mode, integer reference millivolts,
and board timestamp. The Uno does not calculate or transmit floating-point
voltage values.

Instrumented `analogWrite()` preserves the Arduino core behavior on every pin.
Calls on the Uno's real hardware PWM pins D3, D5, D6, D9, D10, and D11 report
the requested integer duty count plus a snapshot of the
ATmega328P timer that drives the pin: timer/channel, waveform mode, polarity,
source clock, prescaler, TOP, output compare, counter, and raw control
registers. Duty endpoints are reported as constant LOW or HIGH because Arduino
disconnects the timer output and does not produce a carrier waveform.

The desktop derives the configured pulse period, frequency, HIGH time, LOW
time, and duty from these integer fields. This is substantially more accurate
than a rounded pin-frequency table, but it remains a timer configuration rather
than an electrical voltage measurement.

## Compile the example

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
firmware/ArduinoSignalVisualizer/examples/GpioDemo
```

Compile the Milestone 2 ADC example:

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
firmware/ArduinoSignalVisualizer/examples/AdcDemo
```

Compile the Milestone 3 PWM example:

```powershell
arduino-cli compile --fqbn arduino:avr:uno `
  --library firmware/ArduinoSignalVisualizer `
  firmware/ArduinoSignalVisualizer/examples/PwmDemo
```

Compile the transparent Serial example:

```powershell
platformio run --project-conf platformio-serial.ini `
  -e uno_transparent_serial_demo
```

PlatformIO keeps the two build and upload targets explicit:

```powershell
platformio run -e uno_gpio_demo
platformio run --project-conf platformio-adc.ini -e uno_adc_demo
platformio run --project-conf platformio-pwm.ini -e uno_pwm_demo
```

Pins D0 and D1 carry the Uno's hardware UART. Instrumenting those pins while
the ASV protocol uses USB serial will interfere with communication.

Digital voltage in the desktop UI is an estimate of either 0 V or the nominal
5 V logic supply; ASV does not measure digital-pin voltage.

ADC voltage is also an estimate. It is calculated on the desktop from the raw
count and declared reference metadata; the default 5000 mV value is not a
calibrated measurement of the Uno supply or AREF pin.
