#include <ArduinoSignalVisualizer.h>

void setup() {
  ASV.begin();
  ASV.pinMode(LED_BUILTIN, OUTPUT);
}

void loop() {
  ASV.digitalWrite(LED_BUILTIN, HIGH);
  delay(100);
  ASV.digitalWrite(LED_BUILTIN, LOW);
  delay(100);
}
