#include "ArduinoSignalVisualizer.h"

ArduinoSignalVisualizer ASV;

ArduinoSignalVisualizer::ArduinoSignalVisualizer()
    : transport_(nullptr),
      sequence_(0),
      analogReferenceMode_(0),
      analogReferenceMillivolts_(5000) {
  for (uint8_t pin = 0; pin < kDigitalPinCount; ++pin) {
    pinModes_[pin] = kUnknownMode;
  }
}

void ArduinoSignalVisualizer::begin(unsigned long baud) {
  begin(Serial, baud);
}

void ArduinoSignalVisualizer::begin(HardwareSerial& serial,
                                    unsigned long baud) {
  serial.begin(baud);
  transport_ = &serial;
  sequence_ = 0;
  sendHello();
}

void ArduinoSignalVisualizer::pinMode(uint8_t pin, uint8_t mode) {
  ::pinMode(pin, mode);
  if (pin >= kDigitalPinCount) {
    return;
  }

  pinModes_[pin] = wireMode(mode);
  sendDigital(pin, static_cast<uint8_t>(::digitalRead(pin) == HIGH), 2);
}

void ArduinoSignalVisualizer::digitalWrite(uint8_t pin, uint8_t value) {
  ::digitalWrite(pin, value);
  if (pin < kDigitalPinCount) {
    sendDigital(pin, static_cast<uint8_t>(value == HIGH), 0);
  }
}

int ArduinoSignalVisualizer::digitalRead(uint8_t pin) {
  const int value = ::digitalRead(pin);
  if (pin < kDigitalPinCount) {
    sendDigital(pin, static_cast<uint8_t>(value == HIGH), 1);
  }
  return value;
}

void ArduinoSignalVisualizer::analogReference(uint8_t mode) {
  const uint8_t wireMode = wireAnalogReferenceMode(mode);
  analogReference(mode, defaultAnalogReferenceMillivolts(wireMode));
}

void ArduinoSignalVisualizer::analogReference(
    uint8_t mode, uint16_t referenceMillivolts) {
  ::analogReference(mode);
  analogReferenceMode_ = wireAnalogReferenceMode(mode);
  analogReferenceMillivolts_ = referenceMillivolts;
}

int ArduinoSignalVisualizer::analogRead(uint8_t pin) {
  const int value = ::analogRead(pin);
  const uint8_t channel = analogChannel(pin);
  if (channel != kUnknownAnalogChannel && value >= 0) {
    sendAnalog(channel, static_cast<uint16_t>(value));
  }
  return value;
}

void ArduinoSignalVisualizer::sendHello() {
  const uint8_t payload[] = {
      1,     // Arduino Uno R3
      0, 2, 0,  // firmware 0.2.0
      3, 0,  // digital GPIO and ADC capabilities
      0,     // reset cause is unknown in the portable v1 implementation
      0x88, 0x13,  // 5000 mV
  };
  sendPacket(asv::kBoardHelloPacket, payload, sizeof(payload));
}

void ArduinoSignalVisualizer::sendDigital(uint8_t pin, uint8_t level,
                                          uint8_t source) {
  const uint8_t payload[] = {pin, pinModes_[pin], level, source};
  sendPacket(asv::kDigitalGpioPacket, payload, sizeof(payload));
}

void ArduinoSignalVisualizer::sendAnalog(uint8_t channel, uint16_t rawValue) {
  const uint8_t payload[] = {
      asv::kAnalogEventVersion,
      channel,
      static_cast<uint8_t>(rawValue & 0xff),
      static_cast<uint8_t>(rawValue >> 8),
      kAdcResolutionBits,
      analogReferenceMode_,
      static_cast<uint8_t>(analogReferenceMillivolts_ & 0xff),
      static_cast<uint8_t>(analogReferenceMillivolts_ >> 8),
  };
  sendPacket(asv::kAnalogSamplePacket, payload, sizeof(payload));
}

void ArduinoSignalVisualizer::sendPacket(uint8_t packetType,
                                         const uint8_t* payload,
                                         size_t payloadLength) {
  if (transport_ == nullptr) {
    return;
  }

  uint8_t frame[asv::kMaximumEncodedFrameLength];
  const size_t frameLength =
      asv::encodePacket(packetType, sequence_, micros(), payload, payloadLength,
                        frame, sizeof(frame));
  if (frameLength == 0) {
    return;
  }
  transport_->write(frame, frameLength);
  ++sequence_;
}

uint8_t ArduinoSignalVisualizer::wireMode(uint8_t arduinoMode) const {
  if (arduinoMode == INPUT) {
    return 0;
  }
  if (arduinoMode == OUTPUT) {
    return 1;
  }
  if (arduinoMode == INPUT_PULLUP) {
    return 2;
  }
  return kUnknownMode;
}

uint8_t ArduinoSignalVisualizer::wireAnalogReferenceMode(
    uint8_t arduinoMode) const {
  if (arduinoMode == DEFAULT) {
    return 0;
  }
  if (arduinoMode == INTERNAL) {
    return 1;
  }
  if (arduinoMode == EXTERNAL) {
    return 2;
  }
  return kUnknownMode;
}

uint16_t ArduinoSignalVisualizer::defaultAnalogReferenceMillivolts(
    uint8_t wireMode) const {
  if (wireMode == 0) {
    return 5000;
  }
  if (wireMode == 1) {
    return 1100;
  }
  return 0;
}

uint8_t ArduinoSignalVisualizer::analogChannel(uint8_t pin) const {
#if defined(analogPinToChannel)
  const uint8_t channel = analogPinToChannel(pin);
#else
  const uint8_t channel = pin >= A0 ? pin - A0 : pin;
#endif
  return channel < kAnalogChannelCount ? channel : kUnknownAnalogChannel;
}
