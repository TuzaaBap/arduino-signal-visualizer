#ifndef ASV_INSTRUMENTED_H
#define ASV_INSTRUMENTED_H

#include "ArduinoSignalVisualizer.h"

// Include this header after third-party library headers. These redirects apply
// to ordinary Arduino API calls in the sketch translation unit; the Arduino
// core and already-compiled libraries remain unchanged.
#define pinMode(pin, mode) ASV.pinMode((pin), (mode))
#define digitalWrite(pin, value) ASV.digitalWrite((pin), (value))
#define digitalRead(pin) ASV.digitalRead((pin))
#define analogReference(...) ASV.analogReference(__VA_ARGS__)
#define analogRead(pin) ASV.analogRead((pin))
#define analogWrite(pin, value) ASV.analogWrite((pin), (value))

#endif
