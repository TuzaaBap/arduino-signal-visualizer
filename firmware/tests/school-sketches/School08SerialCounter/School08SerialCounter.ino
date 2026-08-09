#include <ASVInstrumented.h>

int count = 0;

void setup() {
  Serial.begin(115200);
}

void loop() {
  Serial.print("Count: ");
  Serial.println(count);
  count = count + 1;
  delay(500);
}
