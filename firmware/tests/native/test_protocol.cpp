#include <stdint.h>

#include <fstream>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

#include "../../ArduinoSignalVisualizer/src/AsvProtocol.h"

namespace {

std::vector<uint8_t> readHexVector(const char* path) {
  std::ifstream input(path);
  if (!input) {
    throw std::runtime_error("cannot open shared vector");
  }

  std::vector<uint8_t> bytes;
  std::string token;
  while (input >> token) {
    const unsigned long value = std::stoul(token, nullptr, 16);
    if (value > 0xff) {
      throw std::runtime_error("shared vector contains a non-byte value");
    }
    bytes.push_back(static_cast<uint8_t>(value));
  }
  return bytes;
}

bool matchesVector(const std::vector<uint8_t>& expected, uint8_t packetType,
                   uint16_t sequence, uint32_t timestamp,
                   const uint8_t* payload, size_t payloadLength) {
  uint8_t actual[asv::kMaximumEncodedFrameLength];
  const size_t actualLength =
      asv::encodePacket(packetType, sequence, timestamp, payload, payloadLength,
                        actual, sizeof(actual));
  if (actualLength != expected.size()) {
    return false;
  }
  for (size_t index = 0; index < actualLength; ++index) {
    if (actual[index] != expected[index]) {
      return false;
    }
  }
  return true;
}

}  // namespace

int main(int argc, char** argv) {
  if (argc != 4) {
    std::cerr << "usage: test_protocol <gpio-vector.hex> <adc-vector.hex> "
                 "<pwm-vector.hex>\n";
    return 2;
  }

  const uint8_t gpioPayload[] = {13, 1, 1, 0};
  if (!matchesVector(readHexVector(argv[1]), asv::kDigitalGpioPacket, 0x1234,
                     0x01020304, gpioPayload, sizeof(gpioPayload))) {
    std::cerr << "GPIO vector mismatch\n";
    return 1;
  }

  const uint8_t adcPayload[] = {1, 0, 0, 2, 10, 0, 0x88, 0x13};
  if (!matchesVector(readHexVector(argv[2]), asv::kAnalogSamplePacket, 0x2345,
                     0x11223344, adcPayload, sizeof(adcPayload))) {
    std::cerr << "ADC vector mismatch\n";
    return 1;
  }

  const uint8_t pwmPayload[] = {
      2,    9,    128,  0,    8,    1,    1,    0,
      2,    1,    0x00, 0x24, 0xf4, 0x00, 0x40, 0x00,
      0xff, 0x00, 0x80, 0x00, 0x2a, 0x00, 0x81, 0x03,
  };
  if (!matchesVector(readHexVector(argv[3]), asv::kPwmWritePacket, 0x3456,
                     0x55667788, pwmPayload, sizeof(pwmPayload))) {
    std::cerr << "PWM vector mismatch\n";
    return 1;
  }

  std::cout << "shared GPIO, ADC, and PWM protocol vectors passed\n";
  return 0;
}
