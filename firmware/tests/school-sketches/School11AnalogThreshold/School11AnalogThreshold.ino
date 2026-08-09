#include <ASVInstrumented.h>

int sensorValue = 0;

void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
}

void loop() {
  sensorValue = analogRead(A0);

  if (sensorValue > 512) {
    digitalWrite(13, HIGH);
  } else {
    digitalWrite(13, LOW);
  }

  Serial.print("Sensor: ");
  Serial.println(sensorValue);
  delay(100);
}
