#include <ASVInstrumented.h>

int sensorValue = 0;

void setup() {
  Serial.begin(115200);
  pinMode(9, OUTPUT);
  pinMode(13, OUTPUT);
}

void loop() {
  sensorValue = analogRead(A0);
  analogWrite(9, sensorValue / 4);

  if (sensorValue > 512) {
    digitalWrite(13, HIGH);
  } else {
    digitalWrite(13, LOW);
  }

  Serial.print("A0 value: ");
  Serial.println(sensorValue);
  delay(100);
}
