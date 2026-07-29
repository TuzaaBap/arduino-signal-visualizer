#ifndef ARDUINO_SIGNAL_VISUALIZER_H
#define ARDUINO_SIGNAL_VISUALIZER_H

#include <Arduino.h>

#include "AsvProtocol.h"

class ArduinoSignalVisualizer {
 public:
  static constexpr unsigned long kDefaultBaud = 115200;

  ArduinoSignalVisualizer();

  void begin(unsigned long baud = kDefaultBaud);
  void begin(HardwareSerial& serial, unsigned long baud = kDefaultBaud);

  void pinMode(uint8_t pin, uint8_t mode);
  void digitalWrite(uint8_t pin, uint8_t value);
  int digitalRead(uint8_t pin);

 private:
  static constexpr uint8_t kDigitalPinCount = 14;
  static constexpr uint8_t kUnknownMode = 0xff;

  Stream* transport_;
  uint16_t sequence_;
  uint8_t pinModes_[kDigitalPinCount];

  void sendHello();
  void sendDigital(uint8_t pin, uint8_t level, uint8_t source);
  void sendPacket(uint8_t packetType, const uint8_t* payload,
                  size_t payloadLength);
  uint8_t wireMode(uint8_t arduinoMode) const;
};

extern ArduinoSignalVisualizer ASV;

#endif

