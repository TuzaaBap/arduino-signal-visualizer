#include <ArduinoSignalVisualizer.h>

constexpr uint8_t kLedPin = LED_BUILTIN;
constexpr unsigned long kTogglePeriodMs = 500;
constexpr unsigned long kWalkingPeriodMs = 125;
constexpr uint8_t kWalkingPins[] = {2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12};
constexpr size_t kWalkingPinCount =
    sizeof(kWalkingPins) / sizeof(kWalkingPins[0]);

unsigned long lastToggleMs = 0;
unsigned long lastWalkingMs = 0;
bool ledHigh = false;
size_t walkingIndex = 0;

void setup() {
  ASV.begin();
  ASV.pinMode(kLedPin, OUTPUT);
  ASV.digitalWrite(kLedPin, LOW);
  ASV.digitalRead(kLedPin);

  for (size_t index = 0; index < kWalkingPinCount; ++index) {
    ASV.pinMode(kWalkingPins[index], OUTPUT);
    ASV.digitalWrite(kWalkingPins[index], LOW);
    ASV.digitalRead(kWalkingPins[index]);
  }
}

void loop() {
  const unsigned long now = millis();

  if (now - lastToggleMs >= kTogglePeriodMs) {
    lastToggleMs = now;
    ledHigh = !ledHigh;
    ASV.digitalWrite(kLedPin, ledHigh ? HIGH : LOW);
    ASV.digitalRead(kLedPin);
  }

  if (now - lastWalkingMs >= kWalkingPeriodMs) {
    lastWalkingMs = now;

    ASV.digitalWrite(kWalkingPins[walkingIndex], LOW);
    ASV.digitalRead(kWalkingPins[walkingIndex]);
    walkingIndex = (walkingIndex + 1) % kWalkingPinCount;
    ASV.digitalWrite(kWalkingPins[walkingIndex], HIGH);
    ASV.digitalRead(kWalkingPins[walkingIndex]);
  }
}
