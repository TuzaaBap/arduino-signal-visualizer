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

`analog-a0-midscale.hex` represents:

- protocol version 1
- ADC sample packet type `0x11`
- sequence `0x2345`
- board timestamp `0x11223344` microseconds
- ADC event schema version 1
- channel A0
- raw value 512
- 10-bit resolution
- default reference mode
- 5,000 mV nominal reference

The decoded CRC is `0x3718`. The final `00` is the frame delimiter.
