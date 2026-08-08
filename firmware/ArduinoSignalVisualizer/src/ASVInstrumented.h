#ifndef ASV_INSTRUMENTED_H
#define ASV_INSTRUMENTED_H

#include "ArduinoSignalVisualizer.h"

// ASV owns the outer Arduino lifecycle only in the instrumented sketch. The
// user's setup() and loop() keep their familiar names in source, while the
// linker sees private user functions called by ASV's lifecycle wrapper.
// This lets the library initialize after the Arduino core and service pending
// telemetry without requiring ASV.begin(), ASV.attach(), or ASV.service() in
// the sketch.
#define setup() asvInstrumentedUserSetup()
#define loop() asvInstrumentedUserLoop()

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
