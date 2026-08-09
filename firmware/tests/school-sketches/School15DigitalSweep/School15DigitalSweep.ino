#include <ASVInstrumented.h>

void setup() {
  for (int pin = 2; pin <= 13; pin++) {
    pinMode(pin, OUTPUT);
  }
}

void loop() {
  for (int pin = 2; pin <= 13; pin++) {
    digitalWrite(pin, HIGH);
    delay(25);
    digitalWrite(pin, LOW);
  }

  for (int pin = 13; pin >= 2; pin--) {
    digitalWrite(pin, HIGH);
    delay(25);
    digitalWrite(pin, LOW);
  }
}
