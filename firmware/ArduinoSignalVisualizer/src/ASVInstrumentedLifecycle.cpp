#include "ArduinoSignalVisualizer.h"

// ASVInstrumented.h renames only the sketch's lifecycle functions to these
// private symbols. Arduino's core still calls setup() and loop(), which are
// supplied here. Keeping this wrapper in its own translation unit means the
// explicit ArduinoSignalVisualizer.h API remains usable without symbol
// conflicts in sketches that define ordinary setup() and loop() themselves.
extern void asvInstrumentedUserSetup() __attribute__((weak));
extern void asvInstrumentedUserLoop() __attribute__((weak));

void setup() __attribute__((weak));
void loop() __attribute__((weak));

void setup() {
  // Start the shared Uno UART at the product default so an empty sketch works.
  // A normal Serial.begin(...) inside the user's setup may select another
  // supported baud rate; the final attach below then sends the ASV hello at
  // that effective UART configuration.
  Serial.begin(ArduinoSignalVisualizer::kDefaultBaud);
  if (asvInstrumentedUserSetup != nullptr) {
    asvInstrumentedUserSetup();
  }
  ASV.attach(Serial);
}

void loop() {
  // Each call is non-blocking and emits at most one pending startup snapshot.
  // Calling on both sides prevents ASV maintenance from changing user timing
  // appreciably while still making progress in ordinary Arduino loops.
  ASV.service();
  if (asvInstrumentedUserLoop != nullptr) {
    asvInstrumentedUserLoop();
  }
  ASV.service();
}
