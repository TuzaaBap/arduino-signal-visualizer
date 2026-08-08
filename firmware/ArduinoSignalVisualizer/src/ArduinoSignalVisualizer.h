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
  void attach(HardwareSerial& serial);

  void pinMode(uint8_t pin, uint8_t mode);
  void digitalWrite(uint8_t pin, uint8_t value);
  int digitalRead(uint8_t pin);
  void analogReference(uint8_t mode);
  void analogReference(uint8_t mode, uint16_t referenceMillivolts);
  int analogRead(uint8_t pin);
  void analogWrite(uint8_t pin, int value);

 private:
  struct PwmTimerSnapshot {
    uint8_t timerNumber;
    uint8_t channel;
    uint8_t waveformMode;
    uint8_t outputPolarity;
    uint32_t timerClockHz;
    uint16_t prescaler;
    uint16_t top;
    uint16_t compareValue;
    uint16_t counterValue;
    uint8_t controlA;
    uint8_t controlB;
  };

  static constexpr uint8_t kDigitalPinCount = 14;
  static constexpr uint8_t kAnalogChannelCount = 6;
  static constexpr uint8_t kAdcResolutionBits = 10;
  static constexpr uint8_t kPwmResolutionBits = 8;
  static constexpr uint8_t kUnknownMode = 0xff;
  static constexpr uint8_t kUnknownAnalogChannel = 0xff;

  HardwareSerial* transport_;
  uint16_t sequence_;
  uint8_t pinModes_[kDigitalPinCount];
  uint8_t analogReferenceMode_;
  uint16_t analogReferenceMillivolts_;

  void sendHello();
  void sendDigital(uint8_t pin, uint8_t level, uint8_t source);
  void sendAnalog(uint8_t channel, uint16_t rawValue);
  void sendPwm(uint8_t pin, uint8_t dutyValue);
  void sendPacket(uint8_t packetType, const uint8_t* payload,
                  size_t payloadLength);
  uint8_t wireMode(uint8_t arduinoMode) const;
  uint8_t wireAnalogReferenceMode(uint8_t arduinoMode) const;
  uint16_t defaultAnalogReferenceMillivolts(uint8_t wireMode) const;
  uint8_t analogChannel(uint8_t pin) const;
  bool isPwmPin(uint8_t pin) const;
  bool readPwmTimerSnapshot(uint8_t pin, PwmTimerSnapshot& snapshot) const;
  uint16_t timer01Prescaler(uint8_t clockSelect) const;
  uint16_t timer2Prescaler(uint8_t clockSelect) const;
  uint8_t outputPolarity(uint8_t controlA, uint8_t channel) const;
};

extern ArduinoSignalVisualizer ASV;

#endif
