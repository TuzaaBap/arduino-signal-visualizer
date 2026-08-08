# ArduinoSignalVisualizer library

This library instruments ordinary Arduino Uno GPIO, ADC, and hardware PWM calls
for the Arduino Signal Visualizer desktop application. It also keeps normal
sketch `Serial.print()`, `Serial.read()`, and `Serial.write()` traffic available
in the application's separate Serial Monitor.

## Install in Arduino IDE 2

1. Download `ArduinoSignalVisualizer-0.4.0.zip` from the matching GitHub beta
   release.
2. In Arduino IDE, select **Sketch > Include Library > Add .ZIP Library...**.
3. Select the downloaded ZIP.
4. Open **File > Examples > ArduinoSignalVisualizer > TransparentSerialDemo**.

The desktop application and library should use the same release version.

## Use in a sketch

Include third-party library headers first, then include the ASV instrumentation
header:

```cpp
#include <ASVInstrumented.h>

void setup() {
  Serial.begin(115200);
  ASV.attach(Serial);
  pinMode(LED_BUILTIN, OUTPUT);
  Serial.println("Normal sketch output");
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

`ASV.attach(Serial)` is the compatibility-first form: the sketch initializes
the UART exactly as it normally would, then gives ASV permission to share it.
For a new sketch, `ASV.begin(115200)` is the shorter equivalent and initializes
`Serial` itself. Select the same baud rate in the desktop connection panel.
Do not initialize the same UART again at a different rate after attaching ASV.

## Serial ownership

The Uno has one hardware UART shared by pins D0/D1 and the USB serial bridge.
Normal sketch text and framed ASV telemetry share that wire. While the desktop
application is connected, use its Serial tab for sketch input and output.
Arduino Serial Monitor, another terminal, and the ASV desktop application
cannot open the same operating-system serial port at the same time.

User Serial traffic has priority. ASV telemetry will be skipped instead of
blocking a sketch when the Uno transmit buffer is full; the desktop reports the
resulting sequence gap. Transparent mode is intended for normal text streams.
An opt-in strict mode will be required later for arbitrary binary streams that
may contain ASV framing delimiters.

This project is independent and is not affiliated with or endorsed by Arduino.
