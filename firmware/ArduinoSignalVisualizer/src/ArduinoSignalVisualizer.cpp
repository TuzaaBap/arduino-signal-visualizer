#include "ArduinoSignalVisualizer.h"

ArduinoSignalVisualizer ASV;

ArduinoSignalVisualizer::ArduinoSignalVisualizer()
    : transport_(nullptr), sequence_(0) {
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

void ArduinoSignalVisualizer::sendHello() {
  const uint8_t payload[] = {
      1,     // Arduino Uno R3
      0, 1, 0,  // firmware 0.1.0
      1, 0,  // digital GPIO capability
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

