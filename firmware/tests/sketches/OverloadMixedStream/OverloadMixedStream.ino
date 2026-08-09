#include <ASVInstrumented.h>

int brightness = 1;
bool ledState = false;
unsigned long loopNumber = 0;

void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
  pinMode(9, OUTPUT);
}

void loop() {
  analogRead(A0);
  analogWrite(9, brightness);
  digitalWrite(13, ledState);

  brightness = brightness + 1;
  if (brightness == 255) {
    brightness = 1;
  }

  ledState = !ledState;
  loopNumber = loopNumber + 1;
  if (loopNumber % 20 == 0) {
    Serial.print("Fast mixed loop ");
    Serial.println(loopNumber);
  }

  delay(1);
}
