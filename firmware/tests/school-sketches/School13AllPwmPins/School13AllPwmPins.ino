#include <ASVInstrumented.h>

int brightness = 32;

void setup() {
  pinMode(3, OUTPUT);
  pinMode(5, OUTPUT);
  pinMode(6, OUTPUT);
  pinMode(9, OUTPUT);
  pinMode(10, OUTPUT);
  pinMode(11, OUTPUT);
}

void loop() {
  analogWrite(3, brightness);
  analogWrite(5, brightness);
  analogWrite(6, brightness);
  analogWrite(9, brightness);
  analogWrite(10, brightness);
  analogWrite(11, brightness);

  brightness = brightness + 32;
  if (brightness > 224) {
    brightness = 32;
  }

  delay(500);
}
