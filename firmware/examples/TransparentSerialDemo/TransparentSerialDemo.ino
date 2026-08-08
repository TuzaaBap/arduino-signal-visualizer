#include <ASVInstrumented.h>

constexpr uint8_t kLedPin = LED_BUILTIN;
constexpr unsigned long kTogglePeriodMs = 1000;

unsigned long lastToggleMs = 0;
unsigned long messageNumber = 0;
bool ledHigh = false;

void setup() {
  ASV.begin(115200);
  pinMode(kLedPin, OUTPUT);
  Serial.println("Transparent Serial and ASV telemetry are both active.");
}

void loop() {
  while (Serial.available() > 0) {
    Serial.write(static_cast<uint8_t>(Serial.read()));
  }

  const unsigned long now = millis();
  if (now - lastToggleMs < kTogglePeriodMs) {
    return;
  }

  lastToggleMs = now;
  ledHigh = !ledHigh;
  digitalWrite(kLedPin, ledHigh ? HIGH : LOW);

  Serial.print("User message ");
  Serial.print(++messageNumber);
  Serial.print(": D13 is ");
  Serial.println(ledHigh ? "HIGH" : "LOW");
}
