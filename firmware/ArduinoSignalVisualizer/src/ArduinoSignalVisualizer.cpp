#include "ArduinoSignalVisualizer.h"

#include <util/atomic.h>

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

void ArduinoSignalVisualizer::analogWrite(uint8_t pin, int value) {
  ::analogWrite(pin, value);
  if (pin >= kDigitalPinCount) {
    return;
  }

  pinModes_[pin] = wireMode(OUTPUT);
  if (isPwmPin(pin) && value >= 0 && value <= 255) {
    sendPwm(pin, static_cast<uint8_t>(value));
  } else if (!isPwmPin(pin)) {
    sendDigital(pin, static_cast<uint8_t>(value < 128 ? LOW : HIGH), 0);
  }
}

void ArduinoSignalVisualizer::sendHello() {
  const uint8_t payload[] = {
      1,     // Arduino Uno R3
      0, 4, 0,  // firmware 0.4.0
      15, 0,  // digital GPIO, ADC, PWM, and transparent Serial capabilities
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

void ArduinoSignalVisualizer::sendPwm(uint8_t pin, uint8_t dutyValue) {
  const bool constantLow = dutyValue == 0;
  const bool constantHigh = dutyValue == 255;
  const uint8_t outputMode = constantLow ? 0 : (constantHigh ? 2 : 1);
  PwmTimerSnapshot timer = {};
  if (!readPwmTimerSnapshot(pin, timer)) {
    return;
  }
  const uint8_t payload[] = {
      asv::kPwmEventVersion,
      pin,
      dutyValue,
      0,
      kPwmResolutionBits,
      outputMode,
      timer.timerNumber,
      timer.channel,
      timer.waveformMode,
      timer.outputPolarity,
      static_cast<uint8_t>(timer.timerClockHz & 0xff),
      static_cast<uint8_t>((timer.timerClockHz >> 8) & 0xff),
      static_cast<uint8_t>((timer.timerClockHz >> 16) & 0xff),
      static_cast<uint8_t>((timer.timerClockHz >> 24) & 0xff),
      static_cast<uint8_t>(timer.prescaler & 0xff),
      static_cast<uint8_t>(timer.prescaler >> 8),
      static_cast<uint8_t>(timer.top & 0xff),
      static_cast<uint8_t>(timer.top >> 8),
      static_cast<uint8_t>(timer.compareValue & 0xff),
      static_cast<uint8_t>(timer.compareValue >> 8),
      static_cast<uint8_t>(timer.counterValue & 0xff),
      static_cast<uint8_t>(timer.counterValue >> 8),
      timer.controlA,
      timer.controlB,
  };
  sendPacket(asv::kPwmWritePacket, payload, sizeof(payload));
}

void ArduinoSignalVisualizer::sendPacket(uint8_t packetType,
                                         const uint8_t* payload,
                                         size_t payloadLength) {
  if (transport_ == nullptr) {
    return;
  }

  uint8_t frame[asv::kMaximumEncodedFrameLength];
  const uint16_t packetSequence = sequence_++;
  const size_t frameLength =
      asv::encodePacket(packetType, packetSequence, micros(), payload, payloadLength,
                        frame, sizeof(frame));
  if (frameLength == 0) {
    return;
  }

  // HardwareSerial has a small fixed transmit buffer on the Uno. Never block
  // the sketch behind instrumentation; a sequence gap tells the desktop that
  // telemetry was skipped while user Serial output had priority.
  if (transport_->availableForWrite() < static_cast<int>(frameLength)) {
    return;
  }
  transport_->write(frame, frameLength);
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

bool ArduinoSignalVisualizer::isPwmPin(uint8_t pin) const {
  return pin == 3 || pin == 5 || pin == 6 || pin == 9 || pin == 10 ||
         pin == 11;
}

bool ArduinoSignalVisualizer::readPwmTimerSnapshot(
    uint8_t pin, PwmTimerSnapshot& snapshot) const {
  snapshot.timerClockHz = F_CPU;

  ATOMIC_BLOCK(ATOMIC_RESTORESTATE) {
    if (pin == 5 || pin == 6) {
      snapshot.timerNumber = 0;
      snapshot.channel = pin == 6 ? 0 : 1;
      snapshot.controlA = TCCR0A;
      snapshot.controlB = TCCR0B;
      snapshot.compareValue = pin == 6 ? OCR0A : OCR0B;
      snapshot.counterValue = TCNT0;
      snapshot.prescaler = timer01Prescaler(snapshot.controlB & 0x07);
      const uint8_t waveform =
          (snapshot.controlA & 0x03) | ((snapshot.controlB & _BV(WGM02)) >> 1);
      snapshot.waveformMode = waveform == 3 || waveform == 7
                                  ? 1
                                  : (waveform == 1 || waveform == 5 ? 2 : 0);
      snapshot.top = waveform == 5 || waveform == 7 ? OCR0A : 0xff;
    } else if (pin == 9 || pin == 10) {
      snapshot.timerNumber = 1;
      snapshot.channel = pin == 9 ? 0 : 1;
      snapshot.controlA = TCCR1A;
      snapshot.controlB = TCCR1B;
      snapshot.compareValue = pin == 9 ? OCR1A : OCR1B;
      snapshot.counterValue = TCNT1;
      snapshot.prescaler = timer01Prescaler(snapshot.controlB & 0x07);
      const uint8_t waveform = (snapshot.controlA & 0x03) |
                               (((snapshot.controlB >> WGM12) & 0x03) << 2);
      if (waveform == 5 || waveform == 6 || waveform == 7 ||
          waveform == 14 || waveform == 15) {
        snapshot.waveformMode = 1;
      } else if (waveform == 1 || waveform == 2 || waveform == 3 ||
                 waveform == 10 || waveform == 11) {
        snapshot.waveformMode = 2;
      } else if (waveform == 8 || waveform == 9) {
        snapshot.waveformMode = 3;
      } else {
        snapshot.waveformMode = 0;
      }
      if (waveform == 1 || waveform == 5) {
        snapshot.top = 0x00ff;
      } else if (waveform == 2 || waveform == 6) {
        snapshot.top = 0x01ff;
      } else if (waveform == 3 || waveform == 7) {
        snapshot.top = 0x03ff;
      } else if (waveform == 8 || waveform == 10 || waveform == 14) {
        snapshot.top = ICR1;
      } else if (waveform == 9 || waveform == 11 || waveform == 15) {
        snapshot.top = OCR1A;
      } else {
        snapshot.top = 0;
      }
    } else {
      snapshot.timerNumber = 2;
      snapshot.channel = pin == 11 ? 0 : 1;
      snapshot.controlA = TCCR2A;
      snapshot.controlB = TCCR2B;
      snapshot.compareValue = pin == 11 ? OCR2A : OCR2B;
      snapshot.counterValue = TCNT2;
      snapshot.prescaler = timer2Prescaler(snapshot.controlB & 0x07);
      const uint8_t waveform =
          (snapshot.controlA & 0x03) | ((snapshot.controlB & _BV(WGM22)) >> 1);
      snapshot.waveformMode = waveform == 3 || waveform == 7
                                  ? 1
                                  : (waveform == 1 || waveform == 5 ? 2 : 0);
      snapshot.top = waveform == 5 || waveform == 7 ? OCR2A : 0xff;
    }
    snapshot.outputPolarity = outputPolarity(snapshot.controlA, snapshot.channel);
  }

  return snapshot.waveformMode != 0 && snapshot.outputPolarity != 0xff &&
         snapshot.prescaler != 0 && snapshot.top != 0 &&
         snapshot.compareValue <= snapshot.top &&
         snapshot.counterValue <= snapshot.top;
}

uint16_t ArduinoSignalVisualizer::timer01Prescaler(
    uint8_t clockSelect) const {
  switch (clockSelect) {
    case 1:
      return 1;
    case 2:
      return 8;
    case 3:
      return 64;
    case 4:
      return 256;
    case 5:
      return 1024;
    default:
      return 0;
  }
}

uint16_t ArduinoSignalVisualizer::timer2Prescaler(
    uint8_t clockSelect) const {
  switch (clockSelect) {
    case 1:
      return 1;
    case 2:
      return 8;
    case 3:
      return 32;
    case 4:
      return 64;
    case 5:
      return 128;
    case 6:
      return 256;
    case 7:
      return 1024;
    default:
      return 0;
  }
}

uint8_t ArduinoSignalVisualizer::outputPolarity(uint8_t controlA,
                                                uint8_t channel) const {
  const uint8_t compareMode = (controlA >> (channel == 0 ? 6 : 4)) & 0x03;
  if (compareMode == 0) {
    return 0;
  }
  if (compareMode == 2) {
    return 1;
  }
  if (compareMode == 3) {
    return 2;
  }
  return 0xff;
}
