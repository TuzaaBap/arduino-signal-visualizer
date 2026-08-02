use super::*;

fn parse_hex(input: &str) -> Vec<u8> {
    input
        .split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).expect("test vector contains valid hex"))
        .collect()
}

fn adc_packet(payload: Vec<u8>) -> Packet {
    Packet {
        packet_type: PacketType::AnalogSample,
        sequence: 0x2345,
        board_timestamp_us: 0x1122_3344,
        payload,
    }
}

#[test]
fn shared_gpio_vector_decodes_and_reencodes() {
    let vector = parse_hex(include_str!(
        "../../../protocol/test-vectors/digital-write-d13-high.hex"
    ));
    let (delimiter, encoded) = vector.split_last().expect("vector is non-empty");
    assert_eq!(*delimiter, 0);

    let packet = decode_frame(encoded).expect("shared vector decodes");
    assert_eq!(packet.sequence, 0x1234);
    assert_eq!(packet.board_timestamp_us, 0x0102_0304);
    assert_eq!(packet.payload, [13, 1, 1, 0]);
    assert_eq!(encode_packet(&packet), vector);

    assert_eq!(
        decode_event(&packet).expect("typed event decodes"),
        ProtocolEvent::DigitalGpio {
            sequence: 0x1234,
            board_timestamp_us: 0x0102_0304,
            pin: 13,
            direction: GpioDirection::Output,
            level: GpioLevel::High,
            source: GpioObservationSource::Write,
        }
    );
}

#[test]
fn shared_adc_vector_decodes_and_reencodes() {
    let vector = parse_hex(include_str!(
        "../../../protocol/test-vectors/analog-a0-midscale.hex"
    ));
    let (delimiter, encoded) = vector.split_last().expect("vector is non-empty");
    assert_eq!(*delimiter, 0);

    let packet = decode_frame(encoded).expect("shared ADC vector decodes");
    assert_eq!(packet.sequence, 0x2345);
    assert_eq!(packet.board_timestamp_us, 0x1122_3344);
    assert_eq!(packet.payload, [1, 0, 0, 2, 10, 0, 0x88, 0x13]);
    assert_eq!(encode_packet(&packet), vector);
    assert_eq!(
        decode_event(&packet).expect("typed ADC event decodes"),
        ProtocolEvent::AnalogSample {
            sequence: 0x2345,
            board_timestamp_us: 0x1122_3344,
            channel: 0,
            raw_value: 512,
            resolution_bits: 10,
            reference_mode: AdcReferenceMode::Default,
            reference_mv: 5_000,
        }
    );
}

#[test]
fn fragmented_stream_emits_only_at_delimiter() {
    let packet = Packet {
        packet_type: PacketType::DigitalGpio,
        sequence: 4,
        board_timestamp_us: 99,
        payload: vec![7, 0, 1, 1],
    };
    let encoded = encode_packet(&packet);
    let mut decoder = StreamDecoder::default();
    assert!(decoder.push(&encoded[..3]).is_empty());
    let results = decoder.push(&encoded[3..]);
    assert_eq!(results, [Ok(packet)]);
}

#[test]
fn resynchronizing_stream_discards_only_an_initial_partial_frame() {
    let packet = Packet {
        packet_type: PacketType::DigitalGpio,
        sequence: 4,
        board_timestamp_us: 99,
        payload: vec![7, 0, 1, 1],
    };
    let valid = encode_packet(&packet);
    let mut bytes = valid[5..].to_vec();
    bytes.extend_from_slice(&valid);

    let results = StreamDecoder::resynchronizing().push(&bytes);
    assert_eq!(results, [Ok(packet)]);
}

#[test]
fn corrupted_frame_is_rejected_without_losing_the_next_frame() {
    let packet = Packet {
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 2,
        payload: vec![13, 1, 0, 0],
    };
    let valid = encode_packet(&packet);
    let mut corrupt = valid.clone();
    let crc_byte = corrupt.len() - 2;
    corrupt[crc_byte] ^= 0x01;
    corrupt.extend_from_slice(&valid);

    let results = StreamDecoder::default().push(&corrupt);
    assert_eq!(results.len(), 2);
    assert!(matches!(results[0], Err(ProtocolError::CrcMismatch { .. })));
    assert_eq!(results[1], Ok(packet));
}

#[test]
fn sequence_tracker_reports_gap_duplicate_and_reset() {
    let packet = |packet_type, sequence| Packet {
        packet_type,
        sequence,
        board_timestamp_us: 0,
        payload: Vec::new(),
    };
    let mut tracker = SequenceTracker::default();
    assert_eq!(
        tracker.observe(&packet(PacketType::DigitalGpio, 10)),
        SequenceObservation::First
    );
    assert_eq!(
        tracker.observe(&packet(PacketType::DigitalGpio, 12)),
        SequenceObservation::Missing { count: 1 }
    );
    assert_eq!(
        tracker.observe(&packet(PacketType::DigitalGpio, 12)),
        SequenceObservation::Duplicate
    );
    assert_eq!(
        tracker.observe(&packet(PacketType::BoardHello, 0)),
        SequenceObservation::BoardReset
    );
}

#[test]
fn overlong_frame_is_discarded_through_delimiter() {
    let mut bytes = vec![1; MAX_ENCODED_FRAME_LEN + 8];
    bytes.push(0);
    let results = StreamDecoder::default().push(&bytes);
    assert_eq!(results, [Err(ProtocolError::FrameTooLong)]);
}

#[test]
fn unsupported_version_is_rejected_before_payload_decoding() {
    let packet = Packet {
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 2,
        payload: vec![13, 1, 1, 0],
    };
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(&framed[..framed.len() - 1]).expect("valid COBS");
    decoded[0] = PROTOCOL_VERSION + 1;
    let encoded = cobs::encode(&decoded);
    assert_eq!(
        decode_frame(&encoded),
        Err(ProtocolError::UnsupportedVersion(PROTOCOL_VERSION + 1))
    );
}

#[test]
fn declared_length_must_exactly_match_frame() {
    let packet = Packet {
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 2,
        payload: vec![13, 1, 1, 0],
    };
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(&framed[..framed.len() - 1]).expect("valid COBS");
    decoded[8..10].copy_from_slice(&5_u16.to_le_bytes());
    let crc_offset = decoded.len() - CRC_LEN;
    let crc = crc::crc16_ccitt_false(&decoded[..crc_offset]);
    decoded[crc_offset..].copy_from_slice(&crc.to_le_bytes());

    assert_eq!(
        decode_frame(&cobs::encode(&decoded)),
        Err(ProtocolError::UnexpectedPayloadLength {
            declared: 5,
            actual: 4,
        })
    );
}

#[test]
fn malformed_cobs_is_reported() {
    assert_eq!(decode_frame(&[3, 1]), Err(ProtocolError::MalformedCobs));
}

#[test]
fn adc_payload_length_is_exact() {
    let packet = adc_packet(vec![1, 0, 0, 2, 10, 0, 0x88]);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidTypedPayloadLength {
            packet_type: PacketType::AnalogSample,
            expected: 8,
            actual: 7,
        })
    );
}

#[test]
fn unsupported_adc_event_version_is_rejected() {
    let packet = adc_packet(vec![2, 0, 0, 2, 10, 0, 0x88, 0x13]);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::UnsupportedAdcEventVersion(2))
    );
}

#[test]
fn unsupported_adc_resolution_is_rejected() {
    let packet = adc_packet(vec![1, 0, 0, 1, 9, 0, 0x88, 0x13]);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::UnsupportedAdcResolution(9))
    );
}

#[test]
fn invalid_adc_channel_is_rejected() {
    let packet = adc_packet(vec![1, 6, 0, 1, 10, 0, 0x88, 0x13]);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidAdcChannel(6))
    );
}

#[test]
fn adc_raw_count_must_fit_declared_resolution() {
    let packet = adc_packet(vec![1, 0, 0, 4, 10, 0, 0x88, 0x13]);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::AdcRawOutOfRange {
            raw: 1_024,
            maximum: 1_023,
            resolution_bits: 10,
        })
    );
}

#[test]
fn adc_crc_corruption_is_rejected() {
    let packet = adc_packet(vec![1, 0, 0, 2, 10, 0, 0x88, 0x13]);
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(&framed[..framed.len() - 1]).expect("valid COBS");
    let payload_byte = HEADER_LEN + 2;
    decoded[payload_byte] ^= 0x01;

    assert!(matches!(
        decode_frame(&cobs::encode(&decoded)),
        Err(ProtocolError::CrcMismatch { .. })
    ));
}

#[test]
fn adc_sequence_gap_is_reported() {
    let mut tracker = SequenceTracker::default();
    let first = adc_packet(vec![1, 0, 0, 2, 10, 0, 0x88, 0x13]);
    let mut later = first.clone();
    later.sequence = first.sequence.wrapping_add(3);

    assert_eq!(tracker.observe(&first), SequenceObservation::First);
    assert_eq!(
        tracker.observe(&later),
        SequenceObservation::Missing { count: 2 }
    );
}
