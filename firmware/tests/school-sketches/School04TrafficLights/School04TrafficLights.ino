#include <ASVInstrumented.h>

void setup() {
  pinMode(8, OUTPUT);
  pinMode(9, OUTPUT);
  pinMode(10, OUTPUT);
}

void loop() {
  digitalWrite(10, HIGH);
  digitalWrite(9, LOW);
  digitalWrite(8, LOW);
  delay(2000);

  digitalWrite(9, HIGH);
  delay(1000);

  digitalWrite(10, LOW);
  digitalWrite(9, LOW);
  digitalWrite(8, HIGH);
  delay(2000);

  digitalWrite(8, LOW);
  digitalWrite(9, HIGH);
  delay(1000);
}
