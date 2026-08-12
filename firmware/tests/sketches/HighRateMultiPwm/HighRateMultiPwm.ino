#include <ASVInstrumented.h>

const byte pwmPins[] = {3, 5, 6, 9, 10, 11};
byte phase = 0;
bool heartbeat = false;
unsigned long previous = 0;

void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
  for (byte index = 0; index < 6; index++) {
    pinMode(pwmPins[index], OUTPUT);
  }
}

void loop() {
  if (millis() - previous < 27) {
    return;
  }

  previous = millis();
  phase += 9;
  for (byte index = 0; index < 6; index++) {
    analogWrite(pwmPins[index], (byte)(phase + index * 37));
  }
  heartbeat = !heartbeat;
  digitalWrite(13, heartbeat);
  Serial.print("phase=");
  Serial.println(phase);
}
