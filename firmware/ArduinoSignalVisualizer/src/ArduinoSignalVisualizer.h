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
  void service();

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
  uint16_t observedDigitalMask_;
  uint8_t observedAnalogMask_;
  uint16_t observedPwmMask_;
  uint16_t pendingDigitalMask_;
  uint8_t pendingAnalogMask_;
  uint16_t pendingPwmMask_;
  uint16_t lastAnalogValues_[kAnalogChannelCount];
  uint8_t lastPwmValues_[kDigitalPinCount];
  bool helloPending_;

  bool sendHello(bool recordDrop = true);
  bool sendDigital(uint8_t pin, uint8_t level, uint8_t source,
                   bool recordDrop = true);
  bool sendAnalog(uint8_t channel, uint16_t rawValue,
                  bool recordDrop = true);
  bool sendPwm(uint8_t pin, uint8_t dutyValue, bool recordDrop = true);
  bool sendPacket(uint8_t packetType, const uint8_t* payload,
                  size_t payloadLength, bool recordDrop);
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
