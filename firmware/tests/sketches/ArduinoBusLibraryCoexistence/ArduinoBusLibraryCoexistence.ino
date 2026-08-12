#include <Wire.h>
#include <SPI.h>
#include <ASVInstrumented.h>

void setup() {
  Wire.begin();
  SPI.begin();
}

void loop() {
  Wire.beginTransmission(0x3C);
  Wire.write(0x00);
  Wire.endTransmission();

  SPI.beginTransaction(SPISettings(1000000, MSBFIRST, SPI_MODE0));
  SPI.transfer(0x55);
  SPI.endTransaction();

  delay(1000);
}
