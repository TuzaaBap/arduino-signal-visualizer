#include <ASVInstrumented.h>

constexpr uint8_t kLedPin = LED_BUILTIN;
constexpr uint8_t kAnalogPins[] = {A0, A1, A2, A3, A4, A5};
constexpr size_t kAnalogPinCount =
    sizeof(kAnalogPins) / sizeof(kAnalogPins[0]);
constexpr unsigned long kSamplePeriodMs = 50;
constexpr unsigned long kLedPeriodMs = 500;

unsigned long lastSampleMs = 0;
unsigned long lastLedMs = 0;
bool ledHigh = false;

void setup() {
  analogReference(DEFAULT, 5000);
  pinMode(kLedPin, OUTPUT);
  digitalWrite(kLedPin, LOW);
  digitalRead(kLedPin);
}

void loop() {
  const unsigned long now = millis();

  if (now - lastSampleMs >= kSamplePeriodMs) {
    lastSampleMs = now;
    for (size_t index = 0; index < kAnalogPinCount; ++index) {
      analogRead(kAnalogPins[index]);
    }
  }

  if (now - lastLedMs >= kLedPeriodMs) {
    lastLedMs = now;
    ledHigh = !ledHigh;
    digitalWrite(kLedPin, ledHigh ? HIGH : LOW);
    digitalRead(kLedPin);
  }
}
