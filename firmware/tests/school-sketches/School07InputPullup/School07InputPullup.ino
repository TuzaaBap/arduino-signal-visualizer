#include <ASVInstrumented.h>

int buttonState = HIGH;

void setup() {
  pinMode(2, INPUT_PULLUP);
  pinMode(13, OUTPUT);
}

void loop() {
  buttonState = digitalRead(2);

  if (buttonState == LOW) {
    digitalWrite(13, HIGH);
  } else {
    digitalWrite(13, LOW);
  }

  delay(50);
}
