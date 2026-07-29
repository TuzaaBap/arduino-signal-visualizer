# Shared protocol vectors

`digital-write-d13-high.hex` represents:

- protocol version 1
- digital GPIO packet
- sequence `0x1234`
- board timestamp `0x01020304`
- pin D13
- output mode
- high level
- write source

The decoded CRC is `0xE985`. The final `00` is the frame delimiter.

