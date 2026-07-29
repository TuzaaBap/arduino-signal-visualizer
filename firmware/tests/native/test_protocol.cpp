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

}  // namespace

int main(int argc, char** argv) {
  if (argc != 2) {
    std::cerr << "usage: test_protocol <shared-vector.hex>\n";
    return 2;
  }

  const std::vector<uint8_t> expected = readHexVector(argv[1]);
  const uint8_t payload[] = {13, 1, 1, 0};
  uint8_t actual[asv::kMaximumEncodedFrameLength];
  const size_t actualLength =
      asv::encodePacket(asv::kDigitalGpioPacket, 0x1234, 0x01020304, payload,
                        sizeof(payload), actual, sizeof(actual));

  if (actualLength != expected.size()) {
    std::cerr << "frame length mismatch\n";
    return 1;
  }
  for (size_t index = 0; index < actualLength; ++index) {
    if (actual[index] != expected[index]) {
      std::cerr << "frame differs at byte " << index << "\n";
      return 1;
    }
  }

  std::cout << "shared protocol vector passed\n";
  return 0;
}

