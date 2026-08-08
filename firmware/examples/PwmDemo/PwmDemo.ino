#include <ASVInstrumented.h>

namespace {

constexpr uint8_t kPwmPins[] = {3, 5, 6, 9, 10, 11};
constexpr uint8_t kDutyValues[] = {0, 64, 128, 191, 255};
constexpr unsigned long kStepDelayMs = 500;

}  // namespace

void setup() {
  ASV.begin();
}

void loop() {
  for (uint8_t duty : kDutyValues) {
    for (uint8_t pin : kPwmPins) {
      analogWrite(pin, duty);
    }
    delay(kStepDelayMs);
  }
}
