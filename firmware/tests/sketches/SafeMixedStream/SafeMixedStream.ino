#include <ASVInstrumented.h>

bool ledState = false;
int brightness = 0;
int brightnessStep = 16;

unsigned long lastAdcRead = 0;
unsigned long lastPwmWrite = 0;
unsigned long lastLedChange = 0;
unsigned long lastSerialMessage = 0;

void setup() {
  Serial.begin(115200);
  pinMode(13, OUTPUT);
  pinMode(9, OUTPUT);
}

void loop() {
  unsigned long now = millis();

  if (now - lastAdcRead >= 5) {
    lastAdcRead = now;
    analogRead(A0);
  }

  if (now - lastPwmWrite >= 20) {
    lastPwmWrite = now;
    analogWrite(9, brightness);
    brightness = brightness + brightnessStep;
    if (brightness >= 240 || brightness <= 0) {
      brightnessStep = -brightnessStep;
    }
  }

  if (now - lastLedChange >= 100) {
    lastLedChange = now;
    ledState = !ledState;
    digitalWrite(13, ledState);
  }

  if (now - lastSerialMessage >= 100) {
    lastSerialMessage = now;
    Serial.print("School stream running at ");
    Serial.print(now);
    Serial.println(" ms");
  }
}
