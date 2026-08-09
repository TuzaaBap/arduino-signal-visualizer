use super::*;

fn parse_hex(input: &str) -> Vec<u8> {
    input
        .split_whitespace()
        .map(|part| u8::from_str_radix(part, 16).expect("test vector contains valid hex"))
        .collect()
}

fn frame_body(frame: &[u8]) -> &[u8] {
    assert_eq!(frame.last(), Some(&0), "frame has a trailing delimiter");
    if frame.first() == Some(&0) {
        &frame[1..frame.len() - 1]
    } else {
        &frame[..frame.len() - 1]
    }
}

fn adc_packet(payload: Vec<u8>) -> Packet {
    Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::AnalogSample,
        sequence: 0x2345,
        board_timestamp_us: 0x1122_3344,
        payload,
    }
}

fn pwm_packet(payload: Vec<u8>) -> Packet {
    Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::PwmWrite,
        sequence: 0x3456,
        board_timestamp_us: 0x5566_7788,
        payload,
    }
}

fn d9_pwm_payload(duty: u16) -> Vec<u8> {
    let output_mode = if duty == 0 {
        0
    } else if duty == 255 {
        2
    } else {
        1
    };
    let polarity = if output_mode == 1 { 1 } else { 0 };
    vec![
        2,
        9,
        duty as u8,
        (duty >> 8) as u8,
        8,
        output_mode,
        1,
        0,
        2,
        polarity,
        0x00,
        0x24,
        0xf4,
        0x00,
        0x40,
        0x00,
        0xff,
        0x00,
        duty as u8,
        (duty >> 8) as u8,
        0x2a,
        0x00,
        if output_mode == 1 { 0x81 } else { 0x01 },
        0x03,
    ]
}

#[test]
fn shared_gpio_vector_decodes_and_reencodes() {
    let vector = parse_hex(include_str!(
        "../../../protocol/test-vectors/v2-digital-write-d13-high.hex"
    ));
    let packet = decode_frame(frame_body(&vector)).expect("shared vector decodes");
    assert_eq!(packet.protocol_version, PROTOCOL_VERSION);
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
        "../../../protocol/test-vectors/v2-analog-a0-midscale.hex"
    ));
    let packet = decode_frame(frame_body(&vector)).expect("shared ADC vector decodes");
    assert_eq!(packet.protocol_version, PROTOCOL_VERSION);
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
fn shared_pwm_vector_decodes_and_reencodes() {
    let vector = parse_hex(include_str!(
        "../../../protocol/test-vectors/v2-pwm-d9-half-duty.hex"
    ));
    let packet = decode_frame(frame_body(&vector)).expect("shared PWM vector decodes");
    assert_eq!(packet.protocol_version, PROTOCOL_VERSION);
    assert_eq!(packet.sequence, 0x3456);
    assert_eq!(packet.board_timestamp_us, 0x5566_7788);
    assert_eq!(packet.payload, d9_pwm_payload(128));
    assert_eq!(encode_packet(&packet), vector);
    assert_eq!(
        decode_event(&packet).expect("typed PWM event decodes"),
        ProtocolEvent::PwmWrite {
            sequence: 0x3456,
            board_timestamp_us: 0x5566_7788,
            pin: 9,
            duty_value: 128,
            resolution_bits: 8,
            output_mode: PwmOutputMode::HardwarePwm,
            timer_number: 1,
            timer_channel: PwmTimerChannel::A,
            waveform_mode: PwmWaveformMode::PhaseCorrectPwm,
            output_polarity: PwmOutputPolarity::NonInverting,
            timer_clock_hz: 16_000_000,
            prescaler: 64,
            top: 255,
            compare_value: 128,
            counter_value: 42,
            control_a: 0x81,
            control_b: 0x03,
        }
    );

    let timing = derive_pwm_timing(
        PwmOutputMode::HardwarePwm,
        PwmWaveformMode::PhaseCorrectPwm,
        PwmOutputPolarity::NonInverting,
        16_000_000,
        64,
        255,
        128,
    )
    .expect("hardware PWM has periodic timing");
    assert_eq!(timing.period_ns, 2_040_000);
    assert_eq!(timing.high_time_ns, 1_024_000);
    assert_eq!(timing.low_time_ns, 1_016_000);
    assert_eq!(timing.frequency_millihz, 490_196);
    assert_eq!(timing.duty_ppm, 501_961);
}

#[test]
fn fragmented_stream_emits_only_at_delimiter() {
    let packet = Packet {
        protocol_version: PROTOCOL_VERSION,
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
fn protocol_v2_uses_explicit_start_and_end_delimiters() {
    let packet = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 7,
        board_timestamp_us: 42,
        payload: vec![13, 1, 1, 0],
    };

    let frame = encode_packet(&packet);
    assert_eq!(frame.first(), Some(&0));
    assert_eq!(frame.last(), Some(&0));
    assert_eq!(decode_frame(frame_body(&frame)), Ok(packet));
}

#[test]
fn transport_decoder_separates_normal_serial_text_from_v2_packets() {
    let hello = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::BoardHello,
        sequence: 0,
        board_timestamp_us: 10,
        payload: vec![1, 0, 4, 0, 7, 0, 0, 0x88, 0x13],
    };
    let gpio = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 20,
        payload: vec![13, 1, 1, 0],
    };
    let mut decoder = TransportDecoder::default();

    assert_eq!(
        decoder.push(&encode_packet(&hello)),
        [TransportItem::Packet(hello)]
    );
    assert_eq!(
        decoder.push(b"Hello from the sketch\r\n"),
        [TransportItem::UserSerial(
            b"Hello from the sketch\r\n".to_vec()
        )]
    );
    assert_eq!(
        decoder.push(&encode_packet(&gpio)),
        [TransportItem::Packet(gpio)]
    );
}

#[test]
fn transport_decoder_accepts_fragmented_v2_frames() {
    let hello = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::BoardHello,
        sequence: 0,
        board_timestamp_us: 10,
        payload: vec![1, 0, 4, 0, 7, 0, 0, 0x88, 0x13],
    };
    let frame = encode_packet(&hello);
    let mut decoder = TransportDecoder::default();

    assert!(decoder.push(&frame[..5]).is_empty());
    assert_eq!(decoder.push(&frame[5..]), [TransportItem::Packet(hello)]);
}

#[test]
fn transport_decoder_reports_corrupt_v2_frame_and_recovers_user_text() {
    let hello = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::BoardHello,
        sequence: 0,
        board_timestamp_us: 10,
        payload: vec![1, 0, 4, 0, 15, 0, 0, 0x88, 0x13],
    };
    let gpio = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 20,
        payload: vec![13, 1, 1, 0],
    };
    let mut decoder = TransportDecoder::default();
    assert_eq!(
        decoder.push(&encode_packet(&hello)),
        [TransportItem::Packet(hello)]
    );

    let encoded = encode_packet(&gpio);
    let mut decoded = cobs::decode(frame_body(&encoded)).expect("valid test packet");
    decoded[HEADER_LEN] ^= 0x01;
    let mut corrupt = vec![0];
    corrupt.extend_from_slice(&cobs::encode(&decoded));
    corrupt.push(0);
    let items = decoder.push(&corrupt);
    assert_eq!(items.len(), 1);
    assert!(matches!(items[0], TransportItem::ProtocolError(_)));
    assert_eq!(
        decoder.push(b"still visible\r\n"),
        [TransportItem::UserSerial(b"still visible\r\n".to_vec())]
    );
}

#[test]
fn transport_decoder_accepts_back_to_back_v2_packets() {
    let first = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::BoardHello,
        sequence: 0,
        board_timestamp_us: 10,
        payload: vec![1, 0, 4, 0, 15, 0, 0, 0x88, 0x13],
    };
    let second = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 20,
        payload: vec![13, 1, 1, 0],
    };
    let mut wire = encode_packet(&first);
    wire.extend_from_slice(&encode_packet(&second));

    let mut decoder = TransportDecoder::default();
    assert_eq!(
        decoder.push(&wire),
        [TransportItem::Packet(first), TransportItem::Packet(second)]
    );
}

#[test]
fn transport_decoder_keeps_legacy_v1_compatibility() {
    let packet = Packet {
        protocol_version: LEGACY_PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 3,
        board_timestamp_us: 4,
        payload: vec![13, 1, 0, 0],
    };
    let mut decoder = TransportDecoder::default();

    assert_eq!(
        decoder.push(&encode_packet(&packet)),
        [TransportItem::Packet(packet)]
    );
}

#[test]
fn legacy_shared_gpio_vector_still_decodes() {
    let vector = parse_hex(include_str!(
        "../../../protocol/test-vectors/digital-write-d13-high.hex"
    ));
    let packet = decode_frame(frame_body(&vector)).expect("legacy vector decodes");

    assert_eq!(packet.protocol_version, LEGACY_PROTOCOL_VERSION);
    assert_eq!(packet.packet_type, PacketType::DigitalGpio);
    assert_eq!(packet.sequence, 0x1234);
}

#[test]
fn resynchronizing_stream_discards_only_an_initial_partial_frame() {
    let packet = Packet {
        protocol_version: PROTOCOL_VERSION,
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
        protocol_version: PROTOCOL_VERSION,
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
        protocol_version: PROTOCOL_VERSION,
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
fn sequence_tracker_accepts_periodic_hello_beacons() {
    let packet = |packet_type, sequence| Packet {
        protocol_version: PROTOCOL_VERSION,
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
        tracker.observe(&packet(PacketType::BoardHello, 11)),
        SequenceObservation::InOrder
    );
    assert_eq!(
        tracker.observe(&packet(PacketType::AnalogSample, 12)),
        SequenceObservation::InOrder
    );
}

#[test]
fn sequence_tracker_ignores_startup_reset_before_data() {
    let packet = |sequence| Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::BoardHello,
        sequence,
        board_timestamp_us: 0,
        payload: Vec::new(),
    };
    let mut tracker = SequenceTracker::default();
    assert_eq!(tracker.observe(&packet(7)), SequenceObservation::First);
    assert_eq!(tracker.observe(&packet(0)), SequenceObservation::First);
    assert_eq!(tracker.observe(&packet(1)), SequenceObservation::InOrder);
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
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 2,
        payload: vec![13, 1, 1, 0],
    };
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(frame_body(&framed)).expect("valid COBS");
    decoded[4] = PROTOCOL_VERSION + 1;
    let encoded = cobs::encode(&decoded);
    assert_eq!(
        decode_frame(&encoded),
        Err(ProtocolError::UnsupportedVersion(PROTOCOL_VERSION + 1))
    );
}

#[test]
fn declared_length_must_exactly_match_frame() {
    let packet = Packet {
        protocol_version: PROTOCOL_VERSION,
        packet_type: PacketType::DigitalGpio,
        sequence: 1,
        board_timestamp_us: 2,
        payload: vec![13, 1, 1, 0],
    };
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(frame_body(&framed)).expect("valid COBS");
    decoded[12..14].copy_from_slice(&5_u16.to_le_bytes());
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
    let mut decoded = cobs::decode(frame_body(&framed)).expect("valid COBS");
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

#[test]
fn pwm_payload_length_is_exact() {
    let mut payload = d9_pwm_payload(128);
    payload.pop();
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidTypedPayloadLength {
            packet_type: PacketType::PwmWrite,
            expected: 24,
            actual: 23,
        })
    );
}

#[test]
fn unsupported_pwm_event_version_is_rejected() {
    let mut payload = d9_pwm_payload(128);
    payload[0] = 3;
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::UnsupportedPwmEventVersion(3))
    );
}

#[test]
fn non_pwm_uno_pin_is_rejected() {
    let mut payload = d9_pwm_payload(128);
    payload[1] = 8;
    let packet = pwm_packet(payload);
    assert_eq!(decode_event(&packet), Err(ProtocolError::InvalidPwmPin(8)));
}

#[test]
fn unsupported_pwm_resolution_is_rejected() {
    let mut payload = d9_pwm_payload(128);
    payload[4] = 10;
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::UnsupportedPwmResolution(10))
    );
}

#[test]
fn pwm_duty_must_fit_declared_resolution() {
    let mut payload = d9_pwm_payload(128);
    payload[2..4].copy_from_slice(&256_u16.to_le_bytes());
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::PwmDutyOutOfRange {
            duty: 256,
            maximum: 255,
            resolution_bits: 8,
        })
    );
}

#[test]
fn pwm_output_mode_must_match_endpoint_behavior() {
    let mut payload = d9_pwm_payload(0);
    payload[5] = 1;
    payload[9] = 1;
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidPwmModeForDuty {
            duty: 0,
            expected: PwmOutputMode::ConstantLow,
            actual: PwmOutputMode::HardwarePwm,
        })
    );
}

#[test]
fn invalid_pwm_output_mode_is_rejected() {
    let mut payload = d9_pwm_payload(128);
    payload[5] = 3;
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidPwmOutputMode(3))
    );
}

#[test]
fn pwm_timer_must_match_pin() {
    let mut payload = d9_pwm_payload(128);
    payload[6] = 0;
    let packet = pwm_packet(payload);
    assert_eq!(
        decode_event(&packet),
        Err(ProtocolError::InvalidPwmTimer {
            pin: 9,
            expected: 1,
            actual: 0,
        })
    );
}

#[test]
fn pwm_timer_channel_must_match_pin() {
    let mut payload = d9_pwm_payload(128);
    payload[7] = 1;
    assert_eq!(
        decode_event(&pwm_packet(payload)),
        Err(ProtocolError::InvalidPwmTimerChannel {
            pin: 9,
            expected: PwmTimerChannel::A,
            actual: PwmTimerChannel::B,
        })
    );
}

#[test]
fn invalid_pwm_waveform_mode_is_rejected() {
    let mut payload = d9_pwm_payload(128);
    payload[8] = 9;
    assert_eq!(
        decode_event(&pwm_packet(payload)),
        Err(ProtocolError::InvalidPwmWaveformMode(9))
    );
}

#[test]
fn hardware_pwm_requires_connected_output_polarity() {
    let mut payload = d9_pwm_payload(128);
    payload[9] = 0;
    assert_eq!(
        decode_event(&pwm_packet(payload)),
        Err(ProtocolError::InvalidPwmPolarityForOutputMode {
            output_mode: PwmOutputMode::HardwarePwm,
            actual: PwmOutputPolarity::Disconnected,
        })
    );
}

#[test]
fn uno_pwm_clock_and_prescaler_are_validated() {
    let mut bad_clock = d9_pwm_payload(128);
    bad_clock[10..14].copy_from_slice(&8_000_000_u32.to_le_bytes());
    assert_eq!(
        decode_event(&pwm_packet(bad_clock)),
        Err(ProtocolError::InvalidPwmTimerClock(8_000_000))
    );

    let mut bad_prescaler = d9_pwm_payload(128);
    bad_prescaler[14..16].copy_from_slice(&7_u16.to_le_bytes());
    assert_eq!(
        decode_event(&pwm_packet(bad_prescaler)),
        Err(ProtocolError::InvalidPwmPrescaler {
            timer: 1,
            prescaler: 7,
        })
    );
}

#[test]
fn pwm_timer_values_must_be_possible() {
    let mut zero_top = d9_pwm_payload(128);
    zero_top[16..18].copy_from_slice(&0_u16.to_le_bytes());
    assert_eq!(
        decode_event(&pwm_packet(zero_top)),
        Err(ProtocolError::InvalidPwmTop { timer: 1, top: 0 })
    );

    let mut compare_above_top = d9_pwm_payload(128);
    compare_above_top[16..18].copy_from_slice(&100_u16.to_le_bytes());
    assert_eq!(
        decode_event(&pwm_packet(compare_above_top)),
        Err(ProtocolError::InvalidPwmCompare {
            compare_value: 128,
            top: 100,
        })
    );

    let mut counter_above_top = d9_pwm_payload(128);
    counter_above_top[20..22].copy_from_slice(&256_u16.to_le_bytes());
    assert_eq!(
        decode_event(&pwm_packet(counter_above_top)),
        Err(ProtocolError::InvalidPwmCounter {
            counter_value: 256,
            top: 255,
        })
    );
}

#[test]
fn normalized_pwm_fields_must_match_raw_timer_controls() {
    let mut waveform_mismatch = d9_pwm_payload(128);
    waveform_mismatch[22] = 0x80;
    assert_eq!(
        decode_event(&pwm_packet(waveform_mismatch)),
        Err(ProtocolError::PwmControlWaveformMismatch {
            declared: PwmWaveformMode::PhaseCorrectPwm,
            raw_mode: 0,
        })
    );

    let mut polarity_mismatch = d9_pwm_payload(128);
    polarity_mismatch[22] = 0xc1;
    assert_eq!(
        decode_event(&pwm_packet(polarity_mismatch)),
        Err(ProtocolError::PwmControlPolarityMismatch {
            declared: PwmOutputPolarity::NonInverting,
        })
    );

    let mut prescaler_mismatch = d9_pwm_payload(128);
    prescaler_mismatch[23] = 0x02;
    assert_eq!(
        decode_event(&pwm_packet(prescaler_mismatch)),
        Err(ProtocolError::PwmControlPrescalerMismatch { declared: 64 })
    );
}

#[test]
fn constant_pwm_endpoint_has_no_periodic_timing() {
    let packet = pwm_packet(d9_pwm_payload(255));
    assert_eq!(
        decode_event(&packet).expect("constant-high endpoint decodes"),
        ProtocolEvent::PwmWrite {
            sequence: 0x3456,
            board_timestamp_us: 0x5566_7788,
            pin: 9,
            duty_value: 255,
            resolution_bits: 8,
            output_mode: PwmOutputMode::ConstantHigh,
            timer_number: 1,
            timer_channel: PwmTimerChannel::A,
            waveform_mode: PwmWaveformMode::PhaseCorrectPwm,
            output_polarity: PwmOutputPolarity::Disconnected,
            timer_clock_hz: 16_000_000,
            prescaler: 64,
            top: 255,
            compare_value: 255,
            counter_value: 42,
            control_a: 0x01,
            control_b: 0x03,
        }
    );
    assert_eq!(
        derive_pwm_timing(
            PwmOutputMode::ConstantHigh,
            PwmWaveformMode::PhaseCorrectPwm,
            PwmOutputPolarity::Disconnected,
            16_000_000,
            64,
            255,
            255,
        ),
        None
    );
}

#[test]
fn fast_pwm_uses_top_plus_one_timer_ticks() {
    let timing = derive_pwm_timing(
        PwmOutputMode::HardwarePwm,
        PwmWaveformMode::FastPwm,
        PwmOutputPolarity::NonInverting,
        16_000_000,
        64,
        255,
        128,
    )
    .expect("fast PWM has periodic timing");
    assert_eq!(timing.period_ns, 1_024_000);
    assert_eq!(timing.high_time_ns, 512_000);
    assert_eq!(timing.low_time_ns, 512_000);
    assert_eq!(timing.frequency_millihz, 976_563);
    assert_eq!(timing.duty_ppm, 500_000);
}

#[test]
fn pwm_crc_corruption_is_rejected() {
    let packet = pwm_packet(d9_pwm_payload(128));
    let framed = encode_packet(&packet);
    let mut decoded = cobs::decode(frame_body(&framed)).expect("valid COBS");
    decoded[HEADER_LEN + 2] ^= 0x01;

    assert!(matches!(
        decode_frame(&cobs::encode(&decoded)),
        Err(ProtocolError::CrcMismatch { .. })
    ));
}

#[test]
fn pwm_sequence_gap_is_reported() {
    let mut tracker = SequenceTracker::default();
    let first = pwm_packet(d9_pwm_payload(128));
    let mut later = first.clone();
    later.sequence = first.sequence.wrapping_add(4);

    assert_eq!(tracker.observe(&first), SequenceObservation::First);
    assert_eq!(
        tracker.observe(&later),
        SequenceObservation::Missing { count: 3 }
    );
}
