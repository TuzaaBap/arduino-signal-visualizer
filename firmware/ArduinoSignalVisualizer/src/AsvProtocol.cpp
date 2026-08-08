#include "AsvProtocol.h"

namespace asv {
namespace {

void appendU16(uint8_t* output, size_t offset, uint16_t value) {
  output[offset] = static_cast<uint8_t>(value & 0xff);
  output[offset + 1] = static_cast<uint8_t>(value >> 8);
}

void appendU32(uint8_t* output, size_t offset, uint32_t value) {
  output[offset] = static_cast<uint8_t>(value & 0xff);
  output[offset + 1] = static_cast<uint8_t>((value >> 8) & 0xff);
  output[offset + 2] = static_cast<uint8_t>((value >> 16) & 0xff);
  output[offset + 3] = static_cast<uint8_t>((value >> 24) & 0xff);
}

size_t cobsEncode(const uint8_t* input, size_t inputLength, uint8_t* output,
                  size_t outputCapacity) {
  if (outputCapacity == 0) {
    return 0;
  }

  size_t readIndex = 0;
  size_t writeIndex = 1;
  size_t codeIndex = 0;
  uint8_t code = 1;

  while (readIndex < inputLength) {
    if (input[readIndex] == 0) {
      if (codeIndex >= outputCapacity) {
        return 0;
      }
      output[codeIndex] = code;
      codeIndex = writeIndex++;
      code = 1;
      ++readIndex;
    } else {
      if (writeIndex >= outputCapacity) {
        return 0;
      }
      output[writeIndex++] = input[readIndex++];
      ++code;
      if (code == 0xff) {
        output[codeIndex] = code;
        codeIndex = writeIndex++;
        code = 1;
      }
    }
  }

  if (codeIndex >= outputCapacity) {
    return 0;
  }
  output[codeIndex] = code;
  return writeIndex;
}

}  // namespace

uint16_t crc16CcittFalse(const uint8_t* data, size_t length) {
  uint16_t crc = 0xffff;
  for (size_t index = 0; index < length; ++index) {
    crc ^= static_cast<uint16_t>(data[index]) << 8;
    for (uint8_t bit = 0; bit < 8; ++bit) {
      crc = (crc & 0x8000) != 0
                ? static_cast<uint16_t>((crc << 1) ^ 0x1021)
                : static_cast<uint16_t>(crc << 1);
    }
  }
  return crc;
}

size_t encodePacket(uint8_t packetType, uint16_t sequence,
                    uint32_t boardTimestampUs, const uint8_t* payload,
                    size_t payloadLength, uint8_t* output,
                    size_t outputCapacity) {
  if (payloadLength > kMaximumPayloadLength ||
      (payloadLength > 0 && payload == nullptr) || output == nullptr) {
    return 0;
  }

  uint8_t decoded[kMaximumDecodedPacketLength];
  for (size_t index = 0; index < kProtocolMagicLength; ++index) {
    decoded[index] = kProtocolMagic[index];
  }
  decoded[4] = kProtocolVersion;
  decoded[5] = packetType;
  appendU16(decoded, 6, sequence);
  appendU32(decoded, 8, boardTimestampUs);
  appendU16(decoded, 12, static_cast<uint16_t>(payloadLength));
  for (size_t index = 0; index < payloadLength; ++index) {
    decoded[kHeaderLength + index] = payload[index];
  }

  const size_t crcOffset = kHeaderLength + payloadLength;
  appendU16(decoded, crcOffset, crc16CcittFalse(decoded, crcOffset));
  const size_t decodedLength = crcOffset + kCrcLength;
  if (outputCapacity < 3) {
    return 0;
  }
  output[0] = 0;
  const size_t encodedLength =
      cobsEncode(decoded, decodedLength, output + 1, outputCapacity - 2);
  if (encodedLength == 0 || encodedLength + 1 >= outputCapacity) {
    return 0;
  }
  output[encodedLength + 1] = 0;
  return encodedLength + 2;
}

}  // namespace asv
