# ArduinoSignalVisualizer library

This library instruments ordinary Arduino Uno GPIO, ADC, and hardware PWM calls
for the Arduino Signal Visualizer desktop application. It also keeps normal
sketch `Serial.print()`, `Serial.read()`, and `Serial.write()` traffic available
in the application's separate Serial Monitor.

## Install in Arduino IDE 2

1. Download `ArduinoSignalVisualizer-0.6.0.zip` from the stable GitHub release.
2. In Arduino IDE, select **Sketch > Include Library > Add .ZIP Library...**.
3. Select the downloaded ZIP.
4. Open **File > Examples > ArduinoSignalVisualizer > BareMinimum**.

The desktop application and library should use the same release version.

## Use in a sketch

Include third-party library headers first, then include the ASV instrumentation
header:

```cpp
#include <ASVInstrumented.h>

void setup() {
}

void loop() {
}
```

That single include performs the ASV startup and maintenance work. The sketch
continues to use ordinary Arduino functions:

```cpp
#include <ASVInstrumented.h>

void setup() {
  pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  digitalWrite(LED_BUILTIN, HIGH);
  delay(500);
  digitalWrite(LED_BUILTIN, LOW);
  delay(500);
}
```

`pinMode`, `digitalWrite`, `digitalRead`, `analogReference`, `analogRead`, and
`analogWrite` keep their normal Arduino-facing behavior while producing ASV
telemetry. The core `Serial` object is not replaced.

If the sketch does not configure `Serial`, ASV uses 115200 baud. A sketch that
needs another supported speed can use its normal Arduino call; no ASV call is
required:

```cpp
void setup() {
  Serial.begin(9600);
  Serial.println("Normal sketch output");
}
```

Select that same baud rate in the desktop connection panel. ASV attaches after
the user's `setup()` returns, so its hello packet uses the final UART
configuration. The explicit `ArduinoSignalVisualizer.h` API remains available
for advanced integrations that intentionally manage the lifecycle themselves.

Back-to-back GPIO, ADC, and PWM updates are retained in fixed latest-state
buffers if the Uno UART is temporarily full. A fair round-robin scheduler
services those buffers around `loop()` and during ordinary `delay()` calls, so
beginner sketches do not add ASV calls or timing workarounds. Sequence numbers
advance only after a complete frame is accepted by the UART.

The library also sends a low-rate board-identification beacon. This allows the
desktop application to reconnect even when a USB adapter does not reset the
Uno when the port is reopened.

## Serial ownership

The Uno has one hardware UART shared by pins D0/D1 and the USB serial bridge.
Normal sketch text and framed ASV telemetry share that wire. While the desktop
application is connected, use its Serial tab for sketch input and output.
Arduino Serial Monitor, another terminal, and the ASV desktop application
cannot open the same operating-system serial port at the same time.

User Serial traffic has priority. When the Uno transmit buffer is temporarily
full, ASV retains the newest GPIO, ADC, and PWM state without blocking the
sketch or generating a false sequence gap. Transparent mode is intended for
normal text streams. An opt-in strict mode will be required later for arbitrary
binary streams that may contain ASV framing delimiters.

This project is independent and is not affiliated with or endorsed by Arduino.
