#ifndef ASV_PROTOCOL_H
#define ASV_PROTOCOL_H

#include <stddef.h>
#include <stdint.h>

namespace asv {

constexpr uint8_t kProtocolVersion = 1;
constexpr uint8_t kBoardHelloPacket = 0x01;
constexpr uint8_t kDigitalGpioPacket = 0x10;
constexpr uint8_t kAnalogSamplePacket = 0x11;
constexpr uint8_t kAnalogEventVersion = 1;
constexpr size_t kHeaderLength = 10;
constexpr size_t kCrcLength = 2;
constexpr size_t kMaximumPayloadLength = 32;
constexpr size_t kMaximumDecodedPacketLength =
    kHeaderLength + kMaximumPayloadLength + kCrcLength;
constexpr size_t kMaximumEncodedFrameLength =
    kMaximumDecodedPacketLength + 2;

uint16_t crc16CcittFalse(const uint8_t* data, size_t length);

// Returns the complete frame length, including the zero delimiter. Returns zero
// when the payload or output buffer is too large.
size_t encodePacket(uint8_t packetType, uint16_t sequence,
                    uint32_t boardTimestampUs, const uint8_t* payload,
                    size_t payloadLength, uint8_t* output,
                    size_t outputCapacity);

}  // namespace asv

#endif
