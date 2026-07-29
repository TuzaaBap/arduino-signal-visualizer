//! Binary framing and typed events for Arduino Signal Visualizer.

mod cobs;
mod crc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const PROTOCOL_VERSION: u8 = 1;
pub const HEADER_LEN: usize = 10;
pub const CRC_LEN: usize = 2;
pub const MAX_DECODED_PACKET_LEN: usize = 128;
pub const MAX_ENCODED_FRAME_LEN: usize = MAX_DECODED_PACKET_LEN + 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    BoardHello = 0x01,
    DigitalGpio = 0x10,
}

impl TryFrom<u8> for PacketType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::BoardHello),
            0x10 => Ok(Self::DigitalGpio),
            other => Err(ProtocolError::UnknownPacketType(other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
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
}

pub fn encode_packet(packet: &Packet) -> Vec<u8> {
    let mut decoded = Vec::with_capacity(HEADER_LEN + packet.payload.len() + CRC_LEN);
    decoded.push(PROTOCOL_VERSION);
    decoded.push(packet.packet_type as u8);
    decoded.extend_from_slice(&packet.sequence.to_le_bytes());
    decoded.extend_from_slice(&packet.board_timestamp_us.to_le_bytes());
    decoded.extend_from_slice(&(packet.payload.len() as u16).to_le_bytes());
    decoded.extend_from_slice(&packet.payload);
    decoded.extend_from_slice(&crc::crc16_ccitt_false(&decoded).to_le_bytes());

    let mut framed = cobs::encode(&decoded);
    framed.push(0);
    framed
}

pub fn decode_frame(encoded_without_delimiter: &[u8]) -> Result<Packet, ProtocolError> {
    let decoded = cobs::decode(encoded_without_delimiter)?;
    if decoded.len() < HEADER_LEN + CRC_LEN {
        return Err(ProtocolError::PacketTooShort {
            minimum: HEADER_LEN + CRC_LEN,
        });
    }

    if decoded[0] != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(decoded[0]));
    }

    let packet_type = PacketType::try_from(decoded[1])?;
    let sequence = u16::from_le_bytes([decoded[2], decoded[3]]);
    let board_timestamp_us = u32::from_le_bytes([decoded[4], decoded[5], decoded[6], decoded[7]]);
    let declared = u16::from_le_bytes([decoded[8], decoded[9]]) as usize;
    let actual = decoded.len() - HEADER_LEN - CRC_LEN;
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
        packet_type,
        sequence,
        board_timestamp_us,
        payload: decoded[HEADER_LEN..crc_offset].to_vec(),
    })
}

pub fn decode_event(packet: &Packet) -> Result<ProtocolEvent, ProtocolError> {
    match packet.packet_type {
        PacketType::BoardHello => decode_hello(packet),
        PacketType::DigitalGpio => decode_gpio(packet),
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
