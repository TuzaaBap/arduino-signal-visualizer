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

`pwm-d9-half-duty.hex` represents:

- protocol version 1
- PWM write packet type `0x12`
- sequence `0x3456`
- board timestamp `0x55667788` microseconds
- PWM event schema version 2
- hardware PWM pin D9
- requested duty value 128
- 8-bit duty resolution
- hardware PWM output mode
- Timer 1 channel A, phase-correct PWM, non-inverting output
- 16,000,000 Hz source clock, prescaler 64, TOP 255
- OCR1A value 128 and TCNT1 snapshot 42
- TCCR1A `0x81` and TCCR1B `0x03`

The decoded CRC is `0x19A5`. The final `00` is the frame delimiter.
