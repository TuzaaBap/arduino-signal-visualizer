//! Binary framing and typed events for Arduino Signal Visualizer.

mod cobs;
mod crc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const LEGACY_PROTOCOL_VERSION: u8 = 1;
pub const PROTOCOL_VERSION: u8 = 2;
pub const PROTOCOL_MAGIC: [u8; 4] = *b"ASV2";
pub const LEGACY_HEADER_LEN: usize = 10;
pub const HEADER_LEN: usize = 14;
pub const CRC_LEN: usize = 2;
pub const MAX_DECODED_PACKET_LEN: usize = 128;
pub const MAX_ENCODED_FRAME_LEN: usize = MAX_DECODED_PACKET_LEN + 3;
pub const ADC_EVENT_VERSION: u8 = 1;
pub const PWM_EVENT_VERSION: u8 = 2;
pub const UNO_ANALOG_CHANNEL_COUNT: u8 = 6;
pub const MAX_REFERENCE_MV: u16 = 6_000;
pub const UNO_PWM_RESOLUTION_BITS: u8 = 8;
pub const UNO_PWM_PINS: [u8; 6] = [3, 5, 6, 9, 10, 11];
pub const UNO_CPU_CLOCK_HZ: u32 = 16_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    BoardHello = 0x01,
    DigitalGpio = 0x10,
    AnalogSample = 0x11,
    PwmWrite = 0x12,
}

impl TryFrom<u8> for PacketType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::BoardHello),
            0x10 => Ok(Self::DigitalGpio),
            0x11 => Ok(Self::AnalogSample),
            0x12 => Ok(Self::PwmWrite),
            other => Err(ProtocolError::UnknownPacketType(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub protocol_version: u8,
    pub packet_type: PacketType,
    pub sequence: u16,
    pub board_timestamp_us: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpioDirection {
    Input,
    Output,
    InputPullup,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpioLevel {
    Low,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GpioObservationSource {
    Write,
    Read,
    ModeChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AdcReferenceMode {
    Default,
    Internal,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PwmOutputMode {
    ConstantLow,
    HardwarePwm,
    ConstantHigh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PwmTimerChannel {
    A,
    B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PwmWaveformMode {
    FastPwm,
    PhaseCorrectPwm,
    PhaseAndFrequencyCorrectPwm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PwmOutputPolarity {
    Disconnected,
    NonInverting,
    Inverting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PwmTiming {
    pub period_ns: u64,
    pub high_time_ns: u64,
    pub low_time_ns: u64,
    pub frequency_millihz: u64,
    pub duty_ppm: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProtocolEvent {
    BoardHello {
        sequence: u16,
        board_timestamp_us: u32,
        board: BoardDescriptor,
    },
    DigitalGpio {
        sequence: u16,
        board_timestamp_us: u32,
        pin: u8,
        direction: GpioDirection,
        level: GpioLevel,
        source: GpioObservationSource,
    },
    AnalogSample {
        sequence: u16,
        board_timestamp_us: u32,
        channel: u8,
        raw_value: u16,
        resolution_bits: u8,
        reference_mode: AdcReferenceMode,
        reference_mv: u16,
    },
    PwmWrite {
        sequence: u16,
        board_timestamp_us: u32,
        pin: u8,
        duty_value: u16,
        resolution_bits: u8,
        output_mode: PwmOutputMode,
        timer_number: u8,
        timer_channel: PwmTimerChannel,
        waveform_mode: PwmWaveformMode,
        output_polarity: PwmOutputPolarity,
        timer_clock_hz: u32,
        prescaler: u16,
        top: u16,
        compare_value: u16,
        counter_value: u16,
        control_a: u8,
        control_b: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardDescriptor {
    pub board_type: BoardType,
    pub firmware_version: FirmwareVersion,
    pub capabilities: u16,
    pub reset_cause: ResetCause,
    pub nominal_logic_mv: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BoardType {
    ArduinoUnoR3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResetCause {
    Unknown,
    PowerOn,
    External,
    BrownOut,
    Watchdog,
    Software,
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("COBS frame is malformed")]
    MalformedCobs,
    #[error("frame exceeded {MAX_ENCODED_FRAME_LEN} encoded bytes")]
    FrameTooLong,
    #[error("decoded packet is shorter than the {minimum}-byte minimum")]
    PacketTooShort { minimum: usize },
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u8),
    #[error("unknown packet type 0x{0:02x}")]
    UnknownPacketType(u8),
    #[error("declared payload length {declared} does not match actual length {actual}")]
    UnexpectedPayloadLength { declared: usize, actual: usize },
    #[error("CRC mismatch: expected 0x{expected:04x}, received 0x{received:04x}")]
    CrcMismatch { expected: u16, received: u16 },
    #[error("packet type {packet_type:?} requires {expected} payload bytes, received {actual}")]
    InvalidTypedPayloadLength {
        packet_type: PacketType,
        expected: usize,
        actual: usize,
    },
    #[error("invalid {field} value {value}")]
    InvalidField { field: &'static str, value: u8 },
    #[error("unsupported ADC event version {0}")]
    UnsupportedAdcEventVersion(u8),
    #[error("invalid ADC channel {0}; Uno channels are 0 through 5")]
    InvalidAdcChannel(u8),
    #[error("unsupported ADC resolution {0} bits")]
    UnsupportedAdcResolution(u8),
    #[error("ADC raw value {raw} exceeds {maximum} for {resolution_bits}-bit resolution")]
    AdcRawOutOfRange {
        raw: u16,
        maximum: u16,
        resolution_bits: u8,
    },
    #[error("invalid ADC reference voltage {0} mV")]
    InvalidAdcReferenceVoltage(u16),
    #[error("unsupported PWM event version {0}")]
    UnsupportedPwmEventVersion(u8),
    #[error("pin D{0} is not a hardware PWM pin on the Arduino Uno")]
    InvalidPwmPin(u8),
    #[error("unsupported PWM resolution {0} bits")]
    UnsupportedPwmResolution(u8),
    #[error("PWM duty value {duty} exceeds {maximum} for {resolution_bits}-bit resolution")]
    PwmDutyOutOfRange {
        duty: u16,
        maximum: u16,
        resolution_bits: u8,
    },
    #[error("invalid PWM output mode {0}")]
    InvalidPwmOutputMode(u8),
    #[error("PWM output mode {actual:?} does not match duty value {duty}; expected {expected:?}")]
    InvalidPwmModeForDuty {
        duty: u16,
        expected: PwmOutputMode,
        actual: PwmOutputMode,
    },
    #[error("timer {actual} does not drive PWM pin D{pin}; expected timer {expected}")]
    InvalidPwmTimer { pin: u8, expected: u8, actual: u8 },
    #[error("timer channel {actual:?} does not drive PWM pin D{pin}; expected {expected:?}")]
    InvalidPwmTimerChannel {
        pin: u8,
        expected: PwmTimerChannel,
        actual: PwmTimerChannel,
    },
    #[error("invalid PWM waveform mode {0}")]
    InvalidPwmWaveformMode(u8),
    #[error("invalid PWM output polarity {0}")]
    InvalidPwmOutputPolarity(u8),
    #[error("PWM output polarity {actual:?} is incompatible with output mode {output_mode:?}")]
    InvalidPwmPolarityForOutputMode {
        output_mode: PwmOutputMode,
        actual: PwmOutputPolarity,
    },
    #[error("invalid Uno timer clock {0} Hz")]
    InvalidPwmTimerClock(u32),
    #[error("invalid prescaler {prescaler} for timer {timer}")]
    InvalidPwmPrescaler { timer: u8, prescaler: u16 },
    #[error("invalid PWM TOP value {top} for timer {timer}")]
    InvalidPwmTop { timer: u8, top: u16 },
    #[error("PWM compare value {compare_value} exceeds TOP {top}")]
    InvalidPwmCompare { compare_value: u16, top: u16 },
    #[error("PWM counter value {counter_value} exceeds TOP {top}")]
    InvalidPwmCounter { counter_value: u16, top: u16 },
    #[error("raw timer waveform mode {raw_mode} does not match declared {declared:?}")]
    PwmControlWaveformMismatch {
        declared: PwmWaveformMode,
        raw_mode: u8,
    },
    #[error("raw timer output mode does not match declared polarity {declared:?}")]
    PwmControlPolarityMismatch { declared: PwmOutputPolarity },
    #[error("raw timer clock-select bits do not match declared prescaler {declared}")]
    PwmControlPrescalerMismatch { declared: u16 },
}

pub fn encode_packet(packet: &Packet) -> Vec<u8> {
    let header_len = if packet.protocol_version == LEGACY_PROTOCOL_VERSION {
        LEGACY_HEADER_LEN
    } else {
        HEADER_LEN
    };
    let mut decoded = Vec::with_capacity(header_len + packet.payload.len() + CRC_LEN);
    if packet.protocol_version == LEGACY_PROTOCOL_VERSION {
        decoded.push(LEGACY_PROTOCOL_VERSION);
    } else {
        decoded.extend_from_slice(&PROTOCOL_MAGIC);
        decoded.push(PROTOCOL_VERSION);
    }
    decoded.push(packet.packet_type as u8);
    decoded.extend_from_slice(&packet.sequence.to_le_bytes());
    decoded.extend_from_slice(&packet.board_timestamp_us.to_le_bytes());
    decoded.extend_from_slice(&(packet.payload.len() as u16).to_le_bytes());
    decoded.extend_from_slice(&packet.payload);
    decoded.extend_from_slice(&crc::crc16_ccitt_false(&decoded).to_le_bytes());

    let mut framed = cobs::encode(&decoded);
    if packet.protocol_version == PROTOCOL_VERSION {
        framed.insert(0, 0);
    }
    framed.push(0);
    framed
}

pub fn decode_frame(encoded_without_delimiter: &[u8]) -> Result<Packet, ProtocolError> {
    let decoded = cobs::decode(encoded_without_delimiter)?;
    let (
        protocol_version,
        header_len,
        packet_type_offset,
        sequence_offset,
        timestamp_offset,
        length_offset,
    ) = if decoded.starts_with(&PROTOCOL_MAGIC) {
        if decoded.len() < HEADER_LEN + CRC_LEN {
            return Err(ProtocolError::PacketTooShort {
                minimum: HEADER_LEN + CRC_LEN,
            });
        }
        if decoded[4] != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(decoded[4]));
        }
        (PROTOCOL_VERSION, HEADER_LEN, 5, 6, 8, 12)
    } else {
        if decoded.len() < LEGACY_HEADER_LEN + CRC_LEN {
            return Err(ProtocolError::PacketTooShort {
                minimum: LEGACY_HEADER_LEN + CRC_LEN,
            });
        }
        if decoded[0] != LEGACY_PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(decoded[0]));
        }
        (LEGACY_PROTOCOL_VERSION, LEGACY_HEADER_LEN, 1, 2, 4, 8)
    };

    if decoded.len() < header_len + CRC_LEN {
        return Err(ProtocolError::PacketTooShort {
            minimum: header_len + CRC_LEN,
        });
    }

    let packet_type = PacketType::try_from(decoded[packet_type_offset])?;
    let sequence = u16::from_le_bytes([decoded[sequence_offset], decoded[sequence_offset + 1]]);
    let board_timestamp_us = u32::from_le_bytes([
        decoded[timestamp_offset],
        decoded[timestamp_offset + 1],
        decoded[timestamp_offset + 2],
        decoded[timestamp_offset + 3],
    ]);
    let declared =
        u16::from_le_bytes([decoded[length_offset], decoded[length_offset + 1]]) as usize;
    let actual = decoded.len() - header_len - CRC_LEN;
    if declared != actual {
        return Err(ProtocolError::UnexpectedPayloadLength { declared, actual });
    }

    let crc_offset = decoded.len() - CRC_LEN;
    let received = u16::from_le_bytes([decoded[crc_offset], decoded[crc_offset + 1]]);
    let expected = crc::crc16_ccitt_false(&decoded[..crc_offset]);
    if received != expected {
        return Err(ProtocolError::CrcMismatch { expected, received });
    }

    Ok(Packet {
        protocol_version,
        packet_type,
        sequence,
        board_timestamp_us,
        payload: decoded[header_len..crc_offset].to_vec(),
    })
}

pub fn decode_event(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    match packet.packet_type {
        PacketType::BoardHello => decode_hello(packet),
        PacketType::DigitalGpio => decode_gpio(packet),
        PacketType::AnalogSample => decode_analog_sample(packet),
        PacketType::PwmWrite => decode_pwm_write(packet),
    }
}

fn decode_hello(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    require_payload_len(packet, 9)?;
    let board_type = match packet.payload[0] {
        1 => BoardType::ArduinoUnoR3,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "board type",
                value,
            });
        }
    };
    let reset_cause = match packet.payload[6] {
        0 => ResetCause::Unknown,
        1 => ResetCause::PowerOn,
        2 => ResetCause::External,
        3 => ResetCause::BrownOut,
        4 => ResetCause::Watchdog,
        5 => ResetCause::Software,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "reset cause",
                value,
            });
        }
    };

    Ok(ProtocolEvent::BoardHello {
        sequence: packet.sequence,
        board_timestamp_us: packet.board_timestamp_us,
        board: BoardDescriptor {
            board_type,
            firmware_version: FirmwareVersion {
                major: packet.payload[1],
                minor: packet.payload[2],
                patch: packet.payload[3],
            },
            capabilities: u16::from_le_bytes([packet.payload[4], packet.payload[5]]),
            reset_cause,
            nominal_logic_mv: u16::from_le_bytes([packet.payload[7], packet.payload[8]]),
        },
    })
}

fn decode_gpio(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    require_payload_len(packet, 4)?;
    let pin = packet.payload[0];
    if pin > 13 {
        return Err(ProtocolError::InvalidField {
            field: "digital pin",
            value: pin,
        });
    }
    let direction = match packet.payload[1] {
        0 => GpioDirection::Input,
        1 => GpioDirection::Output,
        2 => GpioDirection::InputPullup,
        255 => GpioDirection::Unknown,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "GPIO mode",
                value,
            });
        }
    };
    let level = match packet.payload[2] {
        0 => GpioLevel::Low,
        1 => GpioLevel::High,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "GPIO level",
                value,
            });
        }
    };
    let source = match packet.payload[3] {
        0 => GpioObservationSource::Write,
        1 => GpioObservationSource::Read,
        2 => GpioObservationSource::ModeChange,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "GPIO observation source",
                value,
            });
        }
    };

    Ok(ProtocolEvent::DigitalGpio {
        sequence: packet.sequence,
        board_timestamp_us: packet.board_timestamp_us,
        pin,
        direction,
        level,
        source,
    })
}

fn decode_analog_sample(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    require_payload_len(packet, 8)?;
    if packet.payload[0] != ADC_EVENT_VERSION {
        return Err(ProtocolError::UnsupportedAdcEventVersion(packet.payload[0]));
    }

    let channel = packet.payload[1];
    if channel >= UNO_ANALOG_CHANNEL_COUNT {
        return Err(ProtocolError::InvalidAdcChannel(channel));
    }

    let raw_value = u16::from_le_bytes([packet.payload[2], packet.payload[3]]);
    let resolution_bits = packet.payload[4];
    if !matches!(resolution_bits, 8 | 10 | 12 | 14 | 16) {
        return Err(ProtocolError::UnsupportedAdcResolution(resolution_bits));
    }
    let maximum = if resolution_bits == 16 {
        u16::MAX
    } else {
        (1_u16 << resolution_bits) - 1
    };
    if raw_value > maximum {
        return Err(ProtocolError::AdcRawOutOfRange {
            raw: raw_value,
            maximum,
            resolution_bits,
        });
    }

    let reference_mode = match packet.payload[5] {
        0 => AdcReferenceMode::Default,
        1 => AdcReferenceMode::Internal,
        2 => AdcReferenceMode::External,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "ADC reference mode",
                value,
            });
        }
    };
    let reference_mv = u16::from_le_bytes([packet.payload[6], packet.payload[7]]);
    let reference_is_unknown_external =
        reference_mode == AdcReferenceMode::External && reference_mv == 0;
    if !reference_is_unknown_external && !(1..=MAX_REFERENCE_MV).contains(&reference_mv) {
        return Err(ProtocolError::InvalidAdcReferenceVoltage(reference_mv));
    }

    Ok(ProtocolEvent::AnalogSample {
        sequence: packet.sequence,
        board_timestamp_us: packet.board_timestamp_us,
        channel,
        raw_value,
        resolution_bits,
        reference_mode,
        reference_mv,
    })
}

fn decode_pwm_write(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    require_payload_len(packet, 24)?;
    if packet.payload[0] != PWM_EVENT_VERSION {
        return Err(ProtocolError::UnsupportedPwmEventVersion(packet.payload[0]));
    }

    let pin = packet.payload[1];
    if !UNO_PWM_PINS.contains(&pin) {
        return Err(ProtocolError::InvalidPwmPin(pin));
    }

    let duty_value = u16::from_le_bytes([packet.payload[2], packet.payload[3]]);
    let resolution_bits = packet.payload[4];
    if resolution_bits != UNO_PWM_RESOLUTION_BITS {
        return Err(ProtocolError::UnsupportedPwmResolution(resolution_bits));
    }
    let maximum = (1_u16 << resolution_bits) - 1;
    if duty_value > maximum {
        return Err(ProtocolError::PwmDutyOutOfRange {
            duty: duty_value,
            maximum,
            resolution_bits,
        });
    }

    let output_mode = match packet.payload[5] {
        0 => PwmOutputMode::ConstantLow,
        1 => PwmOutputMode::HardwarePwm,
        2 => PwmOutputMode::ConstantHigh,
        value => return Err(ProtocolError::InvalidPwmOutputMode(value)),
    };
    let expected_mode = if duty_value == 0 {
        PwmOutputMode::ConstantLow
    } else if duty_value == maximum {
        PwmOutputMode::ConstantHigh
    } else {
        PwmOutputMode::HardwarePwm
    };
    if output_mode != expected_mode {
        return Err(ProtocolError::InvalidPwmModeForDuty {
            duty: duty_value,
            expected: expected_mode,
            actual: output_mode,
        });
    }

    let timer_number = packet.payload[6];
    let timer_channel = match packet.payload[7] {
        0 => PwmTimerChannel::A,
        1 => PwmTimerChannel::B,
        value => {
            return Err(ProtocolError::InvalidField {
                field: "PWM timer channel",
                value,
            });
        }
    };
    let (expected_timer, expected_channel) = expected_pwm_timer(pin);
    if timer_number != expected_timer {
        return Err(ProtocolError::InvalidPwmTimer {
            pin,
            expected: expected_timer,
            actual: timer_number,
        });
    }
    if timer_channel != expected_channel {
        return Err(ProtocolError::InvalidPwmTimerChannel {
            pin,
            expected: expected_channel,
            actual: timer_channel,
        });
    }

    let waveform_mode = match packet.payload[8] {
        1 => PwmWaveformMode::FastPwm,
        2 => PwmWaveformMode::PhaseCorrectPwm,
        3 => PwmWaveformMode::PhaseAndFrequencyCorrectPwm,
        value => return Err(ProtocolError::InvalidPwmWaveformMode(value)),
    };
    let output_polarity = match packet.payload[9] {
        0 => PwmOutputPolarity::Disconnected,
        1 => PwmOutputPolarity::NonInverting,
        2 => PwmOutputPolarity::Inverting,
        value => return Err(ProtocolError::InvalidPwmOutputPolarity(value)),
    };
    let polarity_is_valid = match output_mode {
        PwmOutputMode::HardwarePwm => output_polarity != PwmOutputPolarity::Disconnected,
        PwmOutputMode::ConstantLow | PwmOutputMode::ConstantHigh => {
            output_polarity == PwmOutputPolarity::Disconnected
        }
    };
    if !polarity_is_valid {
        return Err(ProtocolError::InvalidPwmPolarityForOutputMode {
            output_mode,
            actual: output_polarity,
        });
    }

    let timer_clock_hz = u32::from_le_bytes([
        packet.payload[10],
        packet.payload[11],
        packet.payload[12],
        packet.payload[13],
    ]);
    if timer_clock_hz != UNO_CPU_CLOCK_HZ {
        return Err(ProtocolError::InvalidPwmTimerClock(timer_clock_hz));
    }
    let prescaler = u16::from_le_bytes([packet.payload[14], packet.payload[15]]);
    let valid_prescaler = match timer_number {
        2 => matches!(prescaler, 1 | 8 | 32 | 64 | 128 | 256 | 1024),
        _ => matches!(prescaler, 1 | 8 | 64 | 256 | 1024),
    };
    if !valid_prescaler {
        return Err(ProtocolError::InvalidPwmPrescaler {
            timer: timer_number,
            prescaler,
        });
    }
    let top = u16::from_le_bytes([packet.payload[16], packet.payload[17]]);
    let top_is_valid = top > 0 && (timer_number == 1 || top <= u8::MAX as u16);
    if !top_is_valid {
        return Err(ProtocolError::InvalidPwmTop {
            timer: timer_number,
            top,
        });
    }
    let compare_value = u16::from_le_bytes([packet.payload[18], packet.payload[19]]);
    if compare_value > top {
        return Err(ProtocolError::InvalidPwmCompare { compare_value, top });
    }
    let counter_value = u16::from_le_bytes([packet.payload[20], packet.payload[21]]);
    if counter_value > top {
        return Err(ProtocolError::InvalidPwmCounter { counter_value, top });
    }
    let control_a = packet.payload[22];
    let control_b = packet.payload[23];
    let raw_waveform_mode = raw_pwm_waveform_mode(timer_number, control_a, control_b);
    if raw_waveform_mode != Some(waveform_mode) {
        return Err(ProtocolError::PwmControlWaveformMismatch {
            declared: waveform_mode,
            raw_mode: raw_pwm_mode_number(timer_number, control_a, control_b),
        });
    }
    if raw_pwm_output_polarity(control_a, timer_channel) != Some(output_polarity) {
        return Err(ProtocolError::PwmControlPolarityMismatch {
            declared: output_polarity,
        });
    }
    if raw_pwm_prescaler(timer_number, control_b) != Some(prescaler) {
        return Err(ProtocolError::PwmControlPrescalerMismatch {
            declared: prescaler,
        });
    }

    Ok(ProtocolEvent::PwmWrite {
        sequence: packet.sequence,
        board_timestamp_us: packet.board_timestamp_us,
        pin,
        duty_value,
        resolution_bits,
        output_mode,
        timer_number,
        timer_channel,
        waveform_mode,
        output_polarity,
        timer_clock_hz,
        prescaler,
        top,
        compare_value,
        counter_value,
        control_a,
        control_b,
    })
}

fn raw_pwm_mode_number(timer: u8, control_a: u8, control_b: u8) -> u8 {
    if timer == 1 {
        (control_a & 0x03) | (((control_b >> 3) & 0x03) << 2)
    } else {
        (control_a & 0x03) | ((control_b & 0x08) >> 1)
    }
}

fn raw_pwm_waveform_mode(timer: u8, control_a: u8, control_b: u8) -> Option<PwmWaveformMode> {
    let mode = raw_pwm_mode_number(timer, control_a, control_b);
    if timer == 1 {
        match mode {
            5 | 6 | 7 | 14 | 15 => Some(PwmWaveformMode::FastPwm),
            1 | 2 | 3 | 10 | 11 => Some(PwmWaveformMode::PhaseCorrectPwm),
            8 | 9 => Some(PwmWaveformMode::PhaseAndFrequencyCorrectPwm),
            _ => None,
        }
    } else {
        match mode {
            3 | 7 => Some(PwmWaveformMode::FastPwm),
            1 | 5 => Some(PwmWaveformMode::PhaseCorrectPwm),
            _ => None,
        }
    }
}

fn raw_pwm_output_polarity(control_a: u8, channel: PwmTimerChannel) -> Option<PwmOutputPolarity> {
    let shift = match channel {
        PwmTimerChannel::A => 6,
        PwmTimerChannel::B => 4,
    };
    match (control_a >> shift) & 0x03 {
        0 => Some(PwmOutputPolarity::Disconnected),
        2 => Some(PwmOutputPolarity::NonInverting),
        3 => Some(PwmOutputPolarity::Inverting),
        _ => None,
    }
}

fn raw_pwm_prescaler(timer: u8, control_b: u8) -> Option<u16> {
    let clock_select = control_b & 0x07;
    if timer == 2 {
        match clock_select {
            1 => Some(1),
            2 => Some(8),
            3 => Some(32),
            4 => Some(64),
            5 => Some(128),
            6 => Some(256),
            7 => Some(1024),
            _ => None,
        }
    } else {
        match clock_select {
            1 => Some(1),
            2 => Some(8),
            3 => Some(64),
            4 => Some(256),
            5 => Some(1024),
            _ => None,
        }
    }
}

fn expected_pwm_timer(pin: u8) -> (u8, PwmTimerChannel) {
    match pin {
        3 => (2, PwmTimerChannel::B),
        5 => (0, PwmTimerChannel::B),
        6 => (0, PwmTimerChannel::A),
        9 => (1, PwmTimerChannel::A),
        10 => (1, PwmTimerChannel::B),
        11 => (2, PwmTimerChannel::A),
        _ => unreachable!("pin is validated before timer mapping"),
    }
}

pub fn derive_pwm_timing(
    output_mode: PwmOutputMode,
    waveform_mode: PwmWaveformMode,
    output_polarity: PwmOutputPolarity,
    timer_clock_hz: u32,
    prescaler: u16,
    top: u16,
    compare_value: u16,
) -> Option<PwmTiming> {
    if output_mode != PwmOutputMode::HardwarePwm {
        return None;
    }

    let period_ticks = match waveform_mode {
        PwmWaveformMode::FastPwm => u64::from(top) + 1,
        PwmWaveformMode::PhaseCorrectPwm | PwmWaveformMode::PhaseAndFrequencyCorrectPwm => {
            u64::from(top) * 2
        }
    };
    let non_inverting_high_ticks = match waveform_mode {
        PwmWaveformMode::FastPwm => u64::from(compare_value),
        PwmWaveformMode::PhaseCorrectPwm | PwmWaveformMode::PhaseAndFrequencyCorrectPwm => {
            u64::from(compare_value) * 2
        }
    };
    let high_ticks = match output_polarity {
        PwmOutputPolarity::NonInverting => non_inverting_high_ticks,
        PwmOutputPolarity::Inverting => period_ticks - non_inverting_high_ticks,
        PwmOutputPolarity::Disconnected => return None,
    };
    let low_ticks = period_ticks - high_ticks;
    let timer_denominator = u64::from(timer_clock_hz);
    let tick_scale = u64::from(prescaler) * 1_000_000_000;
    let period_ns = rounded_ratio(period_ticks * tick_scale, timer_denominator);
    let high_time_ns = rounded_ratio(high_ticks * tick_scale, timer_denominator);
    let low_time_ns = rounded_ratio(low_ticks * tick_scale, timer_denominator);
    let frequency_millihz = rounded_ratio(
        u64::from(timer_clock_hz) * 1_000,
        u64::from(prescaler) * period_ticks,
    );
    let duty_ppm = rounded_ratio(high_ticks * 1_000_000, period_ticks) as u32;

    Some(PwmTiming {
        period_ns,
        high_time_ns,
        low_time_ns,
        frequency_millihz,
        duty_ppm,
    })
}

fn rounded_ratio(numerator: u64, denominator: u64) -> u64 {
    (numerator + denominator / 2) / denominator
}

fn require_payload_len(packet: &Packet, expected: usize) -> Result<(), ProtocolError> {
    if packet.payload.len() == expected {
        Ok(())
    } else {
        Err(ProtocolError::InvalidTypedPayloadLength {
            packet_type: packet.packet_type,
            expected,
            actual: packet.payload.len(),
        })
    }
}

#[derive(Debug, Default)]
pub struct StreamDecoder {
    encoded: Vec<u8>,
    discarding_overlong: bool,
    suppress_initial_fragment_error: bool,
}

impl StreamDecoder {
    /// Creates a decoder for a live byte stream that may begin partway through
    /// an already-transmitted frame. A malformed first fragment is discarded
    /// silently; valid first frames and all later errors are still reported.
    pub fn resynchronizing() -> Self {
        Self {
            suppress_initial_fragment_error: true,
            ..Self::default()
        }
    }

    pub fn push(&mut self, bytes: &[u8]) -> Vec<Result<Packet, ProtocolError>> {
        let mut results = Vec::new();
        for &byte in bytes {
            if byte == 0 {
                if self.discarding_overlong {
                    self.discarding_overlong = false;
                    self.encoded.clear();
                    if !self.suppress_initial_fragment_error {
                        results.push(Err(ProtocolError::FrameTooLong));
                    }
                } else if !self.encoded.is_empty() {
                    let result = decode_frame(&self.encoded);
                    self.encoded.clear();
                    if result.is_ok() || !self.suppress_initial_fragment_error {
                        results.push(result);
                    }
                }
                self.suppress_initial_fragment_error = false;
                continue;
            }

            if self.discarding_overlong {
                continue;
            }
            if self.encoded.len() >= MAX_ENCODED_FRAME_LEN {
                self.encoded.clear();
                self.discarding_overlong = true;
                continue;
            }
            self.encoded.push(byte);
        }
        results
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportItem {
    Packet(Packet),
    UserSerial(Vec<u8>),
    ProtocolError(ProtocolError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransportMode {
    Detecting,
    LegacyFramed,
    Multiplexed,
}

/// Decodes both legacy ASV-only streams and protocol-v2 streams that mix
/// framed ASV telemetry with ordinary, unmodified user serial bytes.
#[derive(Debug)]
pub struct TransportDecoder {
    mode: TransportMode,
    encoded: Vec<u8>,
    in_multiplexed_frame: bool,
    suppress_initial_fragment_error: bool,
}

impl Default for TransportDecoder {
    fn default() -> Self {
        Self {
            mode: TransportMode::Detecting,
            encoded: Vec::new(),
            in_multiplexed_frame: false,
            suppress_initial_fragment_error: true,
        }
    }
}

impl TransportDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Vec<TransportItem> {
        let mut items = Vec::new();
        let mut user_serial = Vec::new();

        for &byte in bytes {
            match self.mode {
                TransportMode::Detecting | TransportMode::LegacyFramed => {
                    self.push_framed_byte(byte, &mut items);
                }
                TransportMode::Multiplexed => {
                    self.push_multiplexed_byte(byte, &mut items, &mut user_serial);
                }
            }
        }

        push_user_serial(&mut items, &mut user_serial);
        items
    }

    fn push_framed_byte(&mut self, byte: u8, items: &mut Vec<TransportItem>) {
        if byte != 0 {
            if self.encoded.len() == MAX_ENCODED_FRAME_LEN {
                self.encoded.clear();
                if !self.suppress_initial_fragment_error {
                    items.push(TransportItem::ProtocolError(ProtocolError::FrameTooLong));
                }
                self.suppress_initial_fragment_error = false;
                return;
            }
            self.encoded.push(byte);
            return;
        }

        if self.encoded.is_empty() {
            return;
        }

        let decoded = decode_frame(&self.encoded);
        self.encoded.clear();
        match decoded {
            Ok(packet) => {
                if self.mode == TransportMode::Detecting {
                    self.mode = if packet.protocol_version == PROTOCOL_VERSION {
                        TransportMode::Multiplexed
                    } else {
                        TransportMode::LegacyFramed
                    };
                }
                items.push(TransportItem::Packet(packet));
            }
            Err(error) if !self.suppress_initial_fragment_error => {
                items.push(TransportItem::ProtocolError(error));
            }
            Err(_) => {}
        }
        self.suppress_initial_fragment_error = false;
    }

    fn push_multiplexed_byte(
        &mut self,
        byte: u8,
        items: &mut Vec<TransportItem>,
        user_serial: &mut Vec<u8>,
    ) {
        if !self.in_multiplexed_frame {
            if byte == 0 {
                push_user_serial(items, user_serial);
                self.in_multiplexed_frame = true;
            } else {
                user_serial.push(byte);
            }
            return;
        }

        if byte != 0 {
            self.encoded.push(byte);
            if self.encoded.len() > MAX_ENCODED_FRAME_LEN {
                user_serial.push(0);
                user_serial.append(&mut self.encoded);
                self.in_multiplexed_frame = false;
            }
            return;
        }

        if self.encoded.is_empty() {
            // Consecutive zero bytes can be the boundary between back-to-back
            // ASV frames. Keep the newest zero as the next opening delimiter.
            self.in_multiplexed_frame = true;
            return;
        }

        let candidate_is_v2 = encoded_candidate_has_v2_magic(&self.encoded);
        let decoded = decode_frame(&self.encoded);
        match decoded {
            Ok(packet) if packet.protocol_version == PROTOCOL_VERSION => {
                push_user_serial(items, user_serial);
                items.push(TransportItem::Packet(packet));
                self.encoded.clear();
                self.in_multiplexed_frame = false;
            }
            Err(error) if candidate_is_v2 => {
                push_user_serial(items, user_serial);
                items.push(TransportItem::ProtocolError(error));
                self.encoded.clear();
                self.in_multiplexed_frame = false;
            }
            _ => {
                user_serial.push(0);
                user_serial.append(&mut self.encoded);
                // The closing zero may also be the opening delimiter of the
                // next ASV frame, so retain it as framing state.
                self.in_multiplexed_frame = true;
            }
        }
    }
}

fn encoded_candidate_has_v2_magic(encoded: &[u8]) -> bool {
    cobs::decode(encoded).is_ok_and(|decoded| decoded.starts_with(&PROTOCOL_MAGIC))
}

fn push_user_serial(items: &mut Vec<TransportItem>, bytes: &mut Vec<u8>) {
    if !bytes.is_empty() {
        items.push(TransportItem::UserSerial(std::mem::take(bytes)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceObservation {
    First,
    InOrder,
    Missing { count: u16 },
    Duplicate,
    OutOfOrder,
    BoardReset,
}

#[derive(Debug, Default)]
pub struct SequenceTracker {
    last: Option<u16>,
}

impl SequenceTracker {
    pub fn observe(&mut self, packet: &Packet) -> SequenceObservation {
        if packet.packet_type == PacketType::BoardHello {
            let observation = if self.last.is_some() {
                SequenceObservation::BoardReset
            } else {
                SequenceObservation::First
            };
            self.last = Some(packet.sequence);
            return observation;
        }

        let Some(last) = self.last else {
            self.last = Some(packet.sequence);
            return SequenceObservation::First;
        };
        let difference = packet.sequence.wrapping_sub(last);
        let observation = match difference {
            0 => SequenceObservation::Duplicate,
            1 => SequenceObservation::InOrder,
            2..=0x7fff => SequenceObservation::Missing {
                count: difference - 1,
            },
            _ => SequenceObservation::OutOfOrder,
        };
        if !matches!(
            observation,
            SequenceObservation::Duplicate | SequenceObservation::OutOfOrder
        ) {
            self.last = Some(packet.sequence);
        }
        observation
    }
}

#[cfg(test)]
mod tests;
