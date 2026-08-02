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
  void analogReference(uint8_t mode);
  void analogReference(uint8_t mode, uint16_t referenceMillivolts);
  int analogRead(uint8_t pin);

 private:
  static constexpr uint8_t kDigitalPinCount = 14;
  static constexpr uint8_t kAnalogChannelCount = 6;
  static constexpr uint8_t kAdcResolutionBits = 10;
  static constexpr uint8_t kUnknownMode = 0xff;
  static constexpr uint8_t kUnknownAnalogChannel = 0xff;

  Stream* transport_;
  uint16_t sequence_;
  uint8_t pinModes_[kDigitalPinCount];
  uint8_t analogReferenceMode_;
  uint16_t analogReferenceMillivolts_;

  void sendHello();
  void sendDigital(uint8_t pin, uint8_t level, uint8_t source);
  void sendAnalog(uint8_t channel, uint16_t rawValue);
  void sendPacket(uint8_t packetType, const uint8_t* payload,
                  size_t payloadLength);
  uint8_t wireMode(uint8_t arduinoMode) const;
  uint8_t wireAnalogReferenceMode(uint8_t arduinoMode) const;
  uint16_t defaultAnalogReferenceMillivolts(uint8_t wireMode) const;
  uint8_t analogChannel(uint8_t pin) const;
};

extern ArduinoSignalVisualizer ASV;

#endif
