#include <ASVInstrumented.h>

void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
}

void loop() {
  digitalWrite(13, HIGH);
  Serial.println("LED is ON");
  delay(1000);

  digitalWrite(13, LOW);
  Serial.println("LED is OFF");
  delay(1000);
}
