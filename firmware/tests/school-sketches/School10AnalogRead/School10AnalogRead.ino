#include <ASVInstrumented.h>

int sensorValue = 0;

void setup() {
  Serial.begin(115200);
}

void loop() {
  sensorValue = analogRead(A0);
  Serial.print("A0: ");
  Serial.println(sensorValue);
  delay(100);
}
