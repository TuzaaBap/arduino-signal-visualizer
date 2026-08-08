use std::{
    collections::{BTreeMap, VecDeque},
    io::{ErrorKind, Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use asv_protocol::{
    BoardDescriptor, GpioDirection, GpioLevel, GpioObservationSource, Packet, PacketType,
    ProtocolEvent, PwmOutputMode, PwmOutputPolarity, PwmTimerChannel, PwmWaveformMode,
    SequenceObservation, SequenceTracker, TransportDecoder, TransportItem, derive_pwm_timing,
};
use serialport::{SerialPort, SerialPortType};
use tauri::{AppHandle, Emitter, State};

use crate::model::{
    AdcBatch, AdcSample, ConnectionMode, ConnectionPhase, ConnectionStatus, DiagnosticCategory,
    GpioBatch, GpioUpdate, ProtocolDiagnostic, PwmBatch, PwmUpdate, SerialActivityBatch,
    SerialPortDescriptor, UserSerialBatch,
};
use crate::validation;

const EVENT_CONNECTION_STATUS: &str = "asv://connection-status";
const EVENT_BOARD_INFO: &str = "asv://board-info";
const EVENT_GPIO_BATCH: &str = "asv://gpio-batch";
const EVENT_SERIAL_ACTIVITY: &str = "asv://serial-activity";
const EVENT_USER_SERIAL: &str = "asv://user-serial";
const EVENT_ADC_BATCH: &str = "asv://adc-batch";
const EVENT_PWM_BATCH: &str = "asv://pwm-batch";
const EVENT_DIAGNOSTIC: &str = "asv://protocol-diagnostic";
const UI_QUEUE_CAPACITY: usize = 256;
const UI_FLUSH_INTERVAL: Duration = Duration::from_millis(33);
const ADC_UI_PENDING_PER_CHANNEL: usize = 64;
const USER_SERIAL_UI_CAPACITY: usize = 8 * 1024;
const HELLO_TIMEOUT: Duration = Duration::from_secs(3);
// Use a short activity envelope so burst boundaries remain visible in the UI.
// This represents observed USB-serial activity, not individual UART bits.
const USB_SERIAL_LED_PULSE_MS: u16 = 100;

type SharedSerialWriter = Arc<Mutex<Box<dyn SerialPort>>>;
type SerialWriterHandle = (SharedSerialWriter, Arc<SerialActivityCounters>);

#[derive(Default)]
pub struct ConnectionManager {
    active: Mutex<Option<ActiveConnection>>,
}

struct ActiveConnection {
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
    serial_writer: Option<SharedSerialWriter>,
    serial_activity: Option<Arc<SerialActivityCounters>>,
}

struct DeliveryContext {
    dropped: Arc<AtomicU64>,
    mode: ConnectionMode,
    port_name: Option<String>,
    serial_activity: Option<Arc<SerialActivityCounters>>,
    dropped_user_serial: Option<Arc<AtomicU64>>,
}

enum SourceMessage {
    Event(ProtocolEvent),
    UserSerial(Vec<u8>),
    Diagnostic(ProtocolDiagnostic),
}

#[derive(Default)]
struct SerialActivityCounters {
    tx_bytes: AtomicU64,
    rx_bytes: AtomicU64,
}

impl SerialActivityCounters {
    fn record_tx(&self, byte_count: usize) {
        self.tx_bytes
            .fetch_add(byte_count as u64, Ordering::Relaxed);
    }

    fn record_rx(&self, byte_count: usize) {
        self.rx_bytes
            .fetch_add(byte_count as u64, Ordering::Relaxed);
    }

    fn take_batch(&self) -> Option<SerialActivityBatch> {
        let tx_bytes = self.tx_bytes.swap(0, Ordering::Relaxed);
        let rx_bytes = self.rx_bytes.swap(0, Ordering::Relaxed);
        if tx_bytes == 0 && rx_bytes == 0 {
            return None;
        }
        Some(SerialActivityBatch {
            tx_bytes,
            rx_bytes,
            pulse_duration_ms: USB_SERIAL_LED_PULSE_MS,
        })
    }
}

#[tauri::command]
pub fn list_serial_ports() -> Result<Vec<SerialPortDescriptor>, String> {
    serialport::available_ports()
        .map_err(|error| format!("Could not enumerate serial ports: {error}"))
        .map(|ports| {
            ports
                .into_iter()
                .map(|port| {
                    let (kind, vendor_id, product_id, manufacturer, product, serial_number) =
                        match port.port_type {
                            SerialPortType::UsbPort(info) => (
                                "usb".to_owned(),
                                Some(info.vid),
                                Some(info.pid),
                                info.manufacturer,
                                info.product,
                                info.serial_number,
                            ),
                            SerialPortType::BluetoothPort => {
                                ("bluetooth".to_owned(), None, None, None, None, None)
                            }
                            SerialPortType::PciPort => {
                                ("pci".to_owned(), None, None, None, None, None)
                            }
                            SerialPortType::Unknown => {
                                ("unknown".to_owned(), None, None, None, None, None)
                            }
                        };
                    SerialPortDescriptor {
                        name: port.port_name,
                        kind,
                        usb_vendor_id: vendor_id,
                        usb_product_id: product_id,
                        manufacturer,
                        product,
                        serial_number,
                    }
                })
                .collect()
        })
}

#[tauri::command]
pub fn connect_serial(
    port_name: String,
    baud_rate: u32,
    app: AppHandle,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    connect_serial_inner(port_name, baud_rate, app, &manager)
}

pub(crate) fn connect_serial_inner(
    port_name: String,
    baud_rate: u32,
    app: AppHandle,
    manager: &ConnectionManager,
) -> Result<(), String> {
    if !(1_200..=2_000_000).contains(&baud_rate) {
        return Err(format!("Unsupported baud rate {baud_rate}"));
    }

    manager.stop_active();
    emit_status(
        &app,
        ConnectionPhase::WaitingForHello,
        Some(ConnectionMode::Serial),
        Some(port_name.clone()),
        format!("Waiting for ASV firmware at {baud_rate} baud"),
    );

    let mut port = match serialport::new(&port_name, baud_rate)
        .timeout(Duration::from_millis(50))
        .open()
    {
        Ok(port) => port,
        Err(error) => {
            let detail = format!(
                "Could not open {port_name}: {error}. Close Arduino IDE Serial Monitor or any other terminal using this port, then reconnect."
            );
            emit_status(
                &app,
                ConnectionPhase::Error,
                Some(ConnectionMode::Serial),
                Some(port_name),
                detail.clone(),
            );
            return Err(detail);
        }
    };
    let _ = port.write_data_terminal_ready(true);

    let active = start_serial_workers(app, port, port_name)?;
    manager.replace(active);
    Ok(())
}

#[tauri::command]
pub fn write_user_serial(
    bytes: Vec<u8>,
    manager: State<'_, ConnectionManager>,
) -> Result<usize, String> {
    if bytes.is_empty() {
        return Ok(0);
    }
    if bytes.len() > 256 {
        return Err("User serial writes are limited to 256 bytes".to_owned());
    }

    let (writer, activity) = manager
        .serial_writer()
        .ok_or_else(|| "No physical serial connection is active".to_owned())?;
    let mut writer = writer
        .lock()
        .map_err(|_| "Serial writer mutex is poisoned".to_owned())?;
    writer
        .write_all(&bytes)
        .map_err(|error| format!("Could not write user serial bytes: {error}"))?;
    activity.record_rx(bytes.len());
    Ok(bytes.len())
}

#[tauri::command]
pub fn start_mock(app: AppHandle, manager: State<'_, ConnectionManager>) -> Result<(), String> {
    manager.stop_active();
    emit_status(
        &app,
        ConnectionPhase::WaitingForHello,
        Some(ConnectionMode::Mock),
        None,
        "Starting deterministic Mock Mode".to_owned(),
    );
    manager.replace(start_mock_workers(app));
    Ok(())
}

#[tauri::command]
pub fn disconnect(app: AppHandle, manager: State<'_, ConnectionManager>) {
    disconnect_inner(app, &manager);
}

pub(crate) fn disconnect_inner(app: AppHandle, manager: &ConnectionManager) {
    manager.stop_active();
    emit_status(
        &app,
        ConnectionPhase::Disconnected,
        None,
        None,
        "Disconnected".to_owned(),
    );
}

impl ConnectionManager {
    fn replace(&self, connection: ActiveConnection) {
        *self.active.lock().expect("connection mutex poisoned") = Some(connection);
    }

    fn stop_active(&self) {
        let active = self
            .active
            .lock()
            .expect("connection mutex poisoned")
            .take();
        if let Some(active) = active {
            active.stop.store(true, Ordering::Relaxed);
            for handle in active.handles {
                let _ = handle.join();
            }
        }
    }

    fn serial_writer(&self) -> Option<SerialWriterHandle> {
        let active = self.active.lock().ok()?;
        Some((
            Arc::clone(active.as_ref()?.serial_writer.as_ref()?),
            Arc::clone(active.as_ref()?.serial_activity.as_ref()?),
        ))
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        if let Ok(active) = self.active.get_mut()
            && let Some(active) = active.take()
        {
            active.stop.store(true, Ordering::Relaxed);
            for handle in active.handles {
                let _ = handle.join();
            }
        }
    }
}

fn start_serial_workers(
    app: AppHandle,
    mut port: Box<dyn SerialPort>,
    port_name: String,
) -> Result<ActiveConnection, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicU64::new(0));
    let dropped_user_serial = Arc::new(AtomicU64::new(0));
    let serial_activity = Arc::new(SerialActivityCounters::default());
    let active_serial_activity = Arc::clone(&serial_activity);
    let serial_writer = port.try_clone().map_err(|error| {
        format!("Could not clone {port_name} for Serial Monitor output: {error}")
    })?;
    let serial_writer = Arc::new(Mutex::new(serial_writer));
    let (sender, receiver) = mpsc::sync_channel(UI_QUEUE_CAPACITY);

    let reader_stop = Arc::clone(&stop);
    let reader_dropped = Arc::clone(&dropped);
    let reader_dropped_user_serial = Arc::clone(&dropped_user_serial);
    let reader_port_name = port_name.clone();
    let reader_app = app.clone();
    let reader_serial_activity = Arc::clone(&serial_activity);
    let reader = thread::spawn(move || {
        let mut bytes = [0_u8; 256];
        let mut decoder = TransportDecoder::default();
        let mut tracker = SequenceTracker::default();
        let mut handshake_complete = false;

        while !reader_stop.load(Ordering::Relaxed) {
            match port.read(&mut bytes) {
                Ok(count) if count > 0 => {
                    // A successful desktop read means the Uno USB bridge transmitted these bytes.
                    reader_serial_activity.record_tx(count);
                    for item in decoder.push(&bytes[..count]) {
                        match item {
                            TransportItem::Packet(packet)
                                if handshake_complete
                                    || packet.packet_type == PacketType::BoardHello =>
                            {
                                handshake_complete = true;
                                handle_packet(
                                    &sender,
                                    &reader_dropped,
                                    &reader_stop,
                                    &mut tracker,
                                    packet,
                                );
                            }
                            TransportItem::Packet(_) => {}
                            TransportItem::UserSerial(bytes) if handshake_complete => {
                                try_send_user_serial(&sender, &reader_dropped_user_serial, bytes);
                            }
                            TransportItem::UserSerial(_) => {}
                            TransportItem::ProtocolError(error) if handshake_complete => try_send(
                                &sender,
                                &reader_dropped,
                                SourceMessage::Diagnostic(ProtocolDiagnostic {
                                    category: DiagnosticCategory::CorruptFrame,
                                    message: error.to_string(),
                                }),
                            ),
                            TransportItem::ProtocolError(_) => {}
                        }
                    }
                }
                Ok(_) => {}
                Err(error)
                    if matches!(error.kind(), ErrorKind::TimedOut | ErrorKind::WouldBlock) => {}
                Err(error) => {
                    emit_status(
                        &reader_app,
                        ConnectionPhase::Error,
                        Some(ConnectionMode::Serial),
                        Some(reader_port_name.clone()),
                        format!("Serial connection to {reader_port_name} ended: {error}"),
                    );
                    reader_stop.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
    });

    let delivery_stop = Arc::clone(&stop);
    let delivery = thread::spawn(move || {
        deliver_events(
            app,
            receiver,
            delivery_stop,
            DeliveryContext {
                dropped,
                mode: ConnectionMode::Serial,
                port_name: Some(port_name),
                serial_activity: Some(serial_activity),
                dropped_user_serial: Some(dropped_user_serial),
            },
        );
    });

    Ok(ActiveConnection {
        stop,
        handles: vec![reader, delivery],
        serial_writer: Some(serial_writer),
        serial_activity: Some(active_serial_activity),
    })
}

fn start_mock_workers(app: AppHandle) -> ActiveConnection {
    let stop = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicU64::new(0));
    let (sender, receiver) = mpsc::sync_channel(UI_QUEUE_CAPACITY);

    let producer_stop = Arc::clone(&stop);
    let producer_dropped = Arc::clone(&dropped);
    let producer = thread::spawn(move || {
        let board = BoardDescriptor {
            board_type: asv_protocol::BoardType::ArduinoUnoR3,
            firmware_version: asv_protocol::FirmwareVersion {
                major: 0,
                minor: 3,
                patch: 0,
            },
            capabilities: 7,
            reset_cause: asv_protocol::ResetCause::Software,
            nominal_logic_mv: 5_000,
        };
        try_send(
            &sender,
            &producer_dropped,
            SourceMessage::Event(ProtocolEvent::BoardHello {
                sequence: 0,
                board_timestamp_us: 0,
                board,
            }),
        );

        let mut sequence = 1_u16;
        let mut tick = 0_u32;
        while !producer_stop.load(Ordering::Relaxed) {
            try_send(
                &sender,
                &producer_dropped,
                SourceMessage::Event(mock_gpio_event(tick, sequence)),
            );
            sequence = sequence.wrapping_add(1);
            if !send_preserved_event(
                &sender,
                &producer_stop,
                SourceMessage::Event(mock_adc_event(tick, sequence)),
            ) {
                break;
            }
            sequence = sequence.wrapping_add(1);
            if !send_preserved_event(
                &sender,
                &producer_stop,
                SourceMessage::Event(mock_pwm_event(tick, sequence)),
            ) {
                break;
            }
            sequence = sequence.wrapping_add(1);
            tick = tick.wrapping_add(1);
            thread::sleep(Duration::from_millis(125));
        }
    });

    let delivery_stop = Arc::clone(&stop);
    let delivery = thread::spawn(move || {
        deliver_events(
            app,
            receiver,
            delivery_stop,
            DeliveryContext {
                dropped,
                mode: ConnectionMode::Mock,
                port_name: None,
                serial_activity: None,
                dropped_user_serial: None,
            },
        );
    });

    ActiveConnection {
        stop,
        handles: vec![producer, delivery],
        serial_writer: None,
        serial_activity: None,
    }
}

fn mock_gpio_event(tick: u32, sequence: u16) -> ProtocolEvent {
    ProtocolEvent::DigitalGpio {
        sequence,
        board_timestamp_us: tick * 125_000,
        pin: 2 + (tick % 12) as u8,
        direction: GpioDirection::Output,
        level: if (tick / 12).is_multiple_of(2) {
            GpioLevel::High
        } else {
            GpioLevel::Low
        },
        source: GpioObservationSource::Write,
    }
}

fn mock_adc_event(tick: u32, sequence: u16) -> ProtocolEvent {
    let position = (tick * 41) % 2_046;
    let raw_value = if position <= 1_023 {
        position
    } else {
        2_046 - position
    } as u16;
    ProtocolEvent::AnalogSample {
        sequence,
        board_timestamp_us: tick * 125_000,
        channel: (tick % 6) as u8,
        raw_value,
        resolution_bits: 10,
        reference_mode: asv_protocol::AdcReferenceMode::Default,
        reference_mv: 5_000,
    }
}

fn mock_pwm_event(tick: u32, sequence: u16) -> ProtocolEvent {
    const PINS: [u8; 6] = [3, 5, 6, 9, 10, 11];
    const DUTIES: [u16; 5] = [0, 64, 128, 191, 255];
    let pin = PINS[(tick as usize) % PINS.len()];
    let duty_value = DUTIES[((tick / PINS.len() as u32) as usize) % DUTIES.len()];
    let output_mode = if duty_value == 0 {
        PwmOutputMode::ConstantLow
    } else if duty_value == 255 {
        PwmOutputMode::ConstantHigh
    } else {
        PwmOutputMode::HardwarePwm
    };
    let (timer_number, timer_channel, waveform_mode, base_control_a, control_b) = match pin {
        3 => (
            2,
            PwmTimerChannel::B,
            PwmWaveformMode::PhaseCorrectPwm,
            0x01,
            0x04,
        ),
        5 => (0, PwmTimerChannel::B, PwmWaveformMode::FastPwm, 0x03, 0x03),
        6 => (0, PwmTimerChannel::A, PwmWaveformMode::FastPwm, 0x03, 0x03),
        9 => (
            1,
            PwmTimerChannel::A,
            PwmWaveformMode::PhaseCorrectPwm,
            0x01,
            0x03,
        ),
        10 => (
            1,
            PwmTimerChannel::B,
            PwmWaveformMode::PhaseCorrectPwm,
            0x01,
            0x03,
        ),
        11 => (
            2,
            PwmTimerChannel::A,
            PwmWaveformMode::PhaseCorrectPwm,
            0x01,
            0x04,
        ),
        _ => unreachable!("mock pin comes from the Uno PWM pin table"),
    };
    let output_polarity = if output_mode == PwmOutputMode::HardwarePwm {
        PwmOutputPolarity::NonInverting
    } else {
        PwmOutputPolarity::Disconnected
    };
    let compare_output_mask = match timer_channel {
        PwmTimerChannel::A => 0x80,
        PwmTimerChannel::B => 0x20,
    };
    ProtocolEvent::PwmWrite {
        sequence,
        board_timestamp_us: tick * 125_000,
        pin,
        duty_value,
        resolution_bits: 8,
        output_mode,
        timer_number,
        timer_channel,
        waveform_mode,
        output_polarity,
        timer_clock_hz: 16_000_000,
        prescaler: 64,
        top: 255,
        compare_value: duty_value,
        counter_value: (tick % 256) as u16,
        control_a: base_control_a
            | if output_mode == PwmOutputMode::HardwarePwm {
                compare_output_mask
            } else {
                0
            },
        control_b,
    }
}

fn handle_packet(
    sender: &SyncSender<SourceMessage>,
    dropped: &AtomicU64,
    stop: &AtomicBool,
    tracker: &mut SequenceTracker,
    packet: Packet,
) {
    let observation = tracker.observe(&packet);
    let diagnostic = match observation {
        SequenceObservation::Missing { count } => Some(ProtocolDiagnostic {
            category: DiagnosticCategory::MissingPackets,
            message: format!("{count} packet(s) were not received"),
        }),
        SequenceObservation::Duplicate => Some(ProtocolDiagnostic {
            category: DiagnosticCategory::DuplicatePacket,
            message: format!("Duplicate packet {}", packet.sequence),
        }),
        SequenceObservation::OutOfOrder => Some(ProtocolDiagnostic {
            category: DiagnosticCategory::OutOfOrderPacket,
            message: format!("Out-of-order packet {}", packet.sequence),
        }),
        SequenceObservation::BoardReset => Some(ProtocolDiagnostic {
            category: DiagnosticCategory::BoardReset,
            message: "Arduino reset detected".to_owned(),
        }),
        SequenceObservation::First | SequenceObservation::InOrder => None,
    };
    if let Some(diagnostic) = diagnostic {
        try_send(sender, dropped, SourceMessage::Diagnostic(diagnostic));
    }
    if matches!(
        observation,
        SequenceObservation::Duplicate | SequenceObservation::OutOfOrder
    ) {
        return;
    }

    match asv_protocol::decode_event(&packet) {
        // D13 drives the Uno's visible L indicator. Preserve its transitions so
        // queue pressure cannot silently change the apparent blink cadence.
        Ok(
            event @ (ProtocolEvent::DigitalGpio { pin: 13, .. }
            | ProtocolEvent::AnalogSample { .. }
            | ProtocolEvent::PwmWrite { .. }),
        ) => {
            let _ = send_preserved_event(sender, stop, SourceMessage::Event(event));
        }
        Ok(event) => try_send(sender, dropped, SourceMessage::Event(event)),
        Err(error) => try_send(
            sender,
            dropped,
            SourceMessage::Diagnostic(ProtocolDiagnostic {
                category: DiagnosticCategory::CorruptFrame,
                message: error.to_string(),
            }),
        ),
    }
}

fn send_preserved_event(
    sender: &SyncSender<SourceMessage>,
    stop: &AtomicBool,
    message: SourceMessage,
) -> bool {
    let mut pending = message;
    loop {
        match sender.try_send(pending) {
            Ok(()) => return true,
            Err(TrySendError::Full(message)) => {
                if stop.load(Ordering::Relaxed) {
                    return false;
                }
                pending = message;
                thread::sleep(Duration::from_millis(1));
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
}

fn try_send(sender: &SyncSender<SourceMessage>, dropped: &AtomicU64, message: SourceMessage) {
    match sender.try_send(message) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

fn try_send_user_serial(
    sender: &SyncSender<SourceMessage>,
    dropped_bytes: &AtomicU64,
    bytes: Vec<u8>,
) {
    let byte_count = bytes.len() as u64;
    match sender.try_send(SourceMessage::UserSerial(bytes)) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => {
            dropped_bytes.fetch_add(byte_count, Ordering::Relaxed);
        }
    }
}

fn deliver_events(
    app: AppHandle,
    receiver: Receiver<SourceMessage>,
    stop: Arc<AtomicBool>,
    context: DeliveryContext,
) {
    let DeliveryContext {
        dropped,
        mode,
        port_name,
        serial_activity,
        dropped_user_serial,
    } = context;
    let started = Instant::now();
    let mut hello_received = false;
    let mut timeout_reported = false;
    let mut latest_gpio = BTreeMap::<u8, GpioUpdate>::new();
    let mut pending_adc = BTreeMap::<u8, VecDeque<AdcSample>>::new();
    let mut coalesced_adc_ui_samples = 0_u64;
    let mut latest_pwm = BTreeMap::<u8, PwmUpdate>::new();
    let mut coalesced_pwm_ui_updates = 0_u64;
    let mut pending_user_serial = VecDeque::<u8>::new();
    let mut evicted_user_serial_bytes = 0_u64;
    let mut next_flush = Instant::now() + UI_FLUSH_INTERVAL;

    while !stop.load(Ordering::Relaxed) {
        loop {
            match receiver.try_recv() {
                Ok(SourceMessage::Event(ProtocolEvent::BoardHello { board, .. })) => {
                    hello_received = true;
                    validation::record_board(&app, board.clone());
                    let _ = app.emit(EVENT_BOARD_INFO, board);
                    emit_status(
                        &app,
                        ConnectionPhase::Connected,
                        Some(mode),
                        port_name.clone(),
                        match mode {
                            ConnectionMode::Serial => "ASV firmware connected".to_owned(),
                            ConnectionMode::Mock => {
                                "Mock Mode — no physical board connected".to_owned()
                            }
                        },
                    );
                }
                Ok(SourceMessage::Event(ProtocolEvent::DigitalGpio {
                    sequence,
                    board_timestamp_us,
                    pin,
                    direction,
                    level,
                    source,
                })) => {
                    latest_gpio.insert(
                        pin,
                        GpioUpdate {
                            sequence,
                            board_timestamp_us,
                            pin,
                            direction,
                            level,
                            source,
                        },
                    );
                }
                Ok(SourceMessage::Event(ProtocolEvent::AnalogSample {
                    sequence,
                    board_timestamp_us,
                    channel,
                    raw_value,
                    resolution_bits,
                    reference_mode,
                    reference_mv,
                })) => {
                    let sample = AdcSample {
                        sequence,
                        board_timestamp_us,
                        channel,
                        raw_value,
                        resolution_bits,
                        reference_mode,
                        reference_mv,
                    };
                    validation::record_adc_sample(&app, sample.clone());
                    let channel_buffer = pending_adc.entry(channel).or_default();
                    if channel_buffer.len() == ADC_UI_PENDING_PER_CHANNEL {
                        channel_buffer.pop_front();
                        coalesced_adc_ui_samples += 1;
                    }
                    channel_buffer.push_back(sample);
                }
                Ok(SourceMessage::Event(ProtocolEvent::PwmWrite {
                    sequence,
                    board_timestamp_us,
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
                })) => {
                    let timing = derive_pwm_timing(
                        output_mode,
                        waveform_mode,
                        output_polarity,
                        timer_clock_hz,
                        prescaler,
                        top,
                        compare_value,
                    );
                    let update = PwmUpdate {
                        sequence,
                        board_timestamp_us,
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
                        period_ns: timing.map(|value| value.period_ns),
                        high_time_ns: timing.map(|value| value.high_time_ns),
                        low_time_ns: timing.map(|value| value.low_time_ns),
                        frequency_millihz: timing.map(|value| value.frequency_millihz),
                        duty_ppm: timing.map(|value| value.duty_ppm).unwrap_or_else(|| {
                            if output_mode == PwmOutputMode::ConstantHigh {
                                1_000_000
                            } else {
                                0
                            }
                        }),
                    };
                    validation::record_pwm_update(&app, update.clone());
                    if latest_pwm.insert(pin, update).is_some() {
                        coalesced_pwm_ui_updates += 1;
                    }
                }
                Ok(SourceMessage::UserSerial(bytes)) => {
                    for byte in bytes {
                        if pending_user_serial.len() == USER_SERIAL_UI_CAPACITY {
                            pending_user_serial.pop_front();
                            evicted_user_serial_bytes += 1;
                        }
                        pending_user_serial.push_back(byte);
                    }
                }
                Ok(SourceMessage::Diagnostic(diagnostic)) => {
                    emit_diagnostic(&app, diagnostic);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        if !hello_received && !timeout_reported && started.elapsed() >= HELLO_TIMEOUT {
            timeout_reported = true;
            emit_status(
                &app,
                ConnectionPhase::Error,
                Some(mode),
                port_name.clone(),
                "No ASV hello packet received. Check the firmware and baud rate.".to_owned(),
            );
        }

        if Instant::now() >= next_flush {
            flush_gpio(&app, &mut latest_gpio, &dropped);
            flush_serial_activity(&app, serial_activity.as_deref());
            flush_user_serial(
                &app,
                &mut pending_user_serial,
                &mut evicted_user_serial_bytes,
                dropped_user_serial.as_deref(),
            );
            flush_adc(&app, &mut pending_adc, &mut coalesced_adc_ui_samples);
            flush_pwm(&app, &mut latest_pwm, &mut coalesced_pwm_ui_updates);
            next_flush = Instant::now() + UI_FLUSH_INTERVAL;
        }
        thread::sleep(Duration::from_millis(4));
    }
    flush_gpio(&app, &mut latest_gpio, &dropped);
    flush_serial_activity(&app, serial_activity.as_deref());
    flush_user_serial(
        &app,
        &mut pending_user_serial,
        &mut evicted_user_serial_bytes,
        dropped_user_serial.as_deref(),
    );
    flush_adc(&app, &mut pending_adc, &mut coalesced_adc_ui_samples);
    flush_pwm(&app, &mut latest_pwm, &mut coalesced_pwm_ui_updates);
}

fn flush_serial_activity(app: &AppHandle, counters: Option<&SerialActivityCounters>) {
    let Some(batch) = counters.and_then(SerialActivityCounters::take_batch) else {
        return;
    };
    let _ = app.emit(EVENT_SERIAL_ACTIVITY, batch);
}

fn flush_user_serial(
    app: &AppHandle,
    pending: &mut VecDeque<u8>,
    evicted_bytes: &mut u64,
    dropped_bytes: Option<&AtomicU64>,
) {
    let dropped = dropped_bytes
        .map(|counter| counter.swap(0, Ordering::Relaxed))
        .unwrap_or(0)
        + std::mem::take(evicted_bytes);
    if pending.is_empty() && dropped == 0 {
        return;
    }
    let batch = UserSerialBatch {
        bytes: pending.drain(..).collect(),
        dropped_bytes: dropped,
    };
    validation::record_user_serial(app, &batch);
    let _ = app.emit(EVENT_USER_SERIAL, batch);
}

fn flush_gpio(app: &AppHandle, latest_gpio: &mut BTreeMap<u8, GpioUpdate>, dropped: &AtomicU64) {
    let dropped_ui_events = dropped.swap(0, Ordering::Relaxed);
    if latest_gpio.is_empty() && dropped_ui_events == 0 {
        return;
    }

    let batch = GpioBatch {
        updates: std::mem::take(latest_gpio).into_values().collect(),
        dropped_ui_events,
    };
    validation::record_batch(app, &batch);
    let _ = app.emit(EVENT_GPIO_BATCH, batch);
    if dropped_ui_events > 0 {
        emit_diagnostic(
            app,
            ProtocolDiagnostic {
                category: DiagnosticCategory::QueuePressure,
                message: format!(
                    "{dropped_ui_events} UI update(s) were coalesced under queue pressure"
                ),
            },
        );
    }
}

fn flush_adc(
    app: &AppHandle,
    pending_adc: &mut BTreeMap<u8, VecDeque<AdcSample>>,
    coalesced_adc_ui_samples: &mut u64,
) {
    if pending_adc.is_empty() && *coalesced_adc_ui_samples == 0 {
        return;
    }

    let samples = std::mem::take(pending_adc)
        .into_values()
        .flat_map(VecDeque::into_iter)
        .collect();
    let batch = AdcBatch {
        samples,
        coalesced_ui_samples: std::mem::take(coalesced_adc_ui_samples),
    };
    let _ = app.emit(EVENT_ADC_BATCH, batch);
}

fn flush_pwm(
    app: &AppHandle,
    latest_pwm: &mut BTreeMap<u8, PwmUpdate>,
    coalesced_pwm_ui_updates: &mut u64,
) {
    if latest_pwm.is_empty() && *coalesced_pwm_ui_updates == 0 {
        return;
    }

    let batch = PwmBatch {
        updates: std::mem::take(latest_pwm).into_values().collect(),
        coalesced_ui_updates: std::mem::take(coalesced_pwm_ui_updates),
    };
    let _ = app.emit(EVENT_PWM_BATCH, batch);
}

fn emit_status(
    app: &AppHandle,
    phase: ConnectionPhase,
    mode: Option<ConnectionMode>,
    port_name: Option<String>,
    detail: String,
) {
    let status = ConnectionStatus {
        phase,
        mode,
        port_name,
        detail,
    };
    validation::record_status(app, status.clone());
    let _ = app.emit(EVENT_CONNECTION_STATUS, status);
}

fn emit_diagnostic(app: &AppHandle, diagnostic: ProtocolDiagnostic) {
    validation::record_diagnostic(app, diagnostic.clone());
    let _ = app.emit(EVENT_DIAGNOSTIC, diagnostic);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serial_activity_is_batched_and_uses_the_ui_pulse_duration() {
        let counters = SerialActivityCounters::default();
        assert_eq!(counters.take_batch(), None);

        counters.record_tx(17);
        assert_eq!(
            counters.take_batch(),
            Some(SerialActivityBatch {
                tx_bytes: 17,
                rx_bytes: 0,
                pulse_duration_ms: 100,
            })
        );
        assert_eq!(counters.take_batch(), None);
    }

    #[test]
    fn mock_source_is_deterministic_and_stays_within_uno_gpio() {
        assert_eq!(
            mock_gpio_event(0, 1),
            ProtocolEvent::DigitalGpio {
                sequence: 1,
                board_timestamp_us: 0,
                pin: 2,
                direction: GpioDirection::Output,
                level: GpioLevel::High,
                source: GpioObservationSource::Write,
            }
        );
        assert_eq!(
            mock_gpio_event(12, 13),
            ProtocolEvent::DigitalGpio {
                sequence: 13,
                board_timestamp_us: 1_500_000,
                pin: 2,
                direction: GpioDirection::Output,
                level: GpioLevel::Low,
                source: GpioObservationSource::Write,
            }
        );
        for tick in 0..1_000 {
            let ProtocolEvent::DigitalGpio { pin, .. } = mock_gpio_event(tick, tick as u16) else {
                panic!("mock source must emit GPIO events");
            };
            assert!((2..=13).contains(&pin));
        }
    }

    #[test]
    fn mock_adc_source_is_deterministic_and_within_declared_resolution() {
        assert_eq!(
            mock_adc_event(0, 2),
            ProtocolEvent::AnalogSample {
                sequence: 2,
                board_timestamp_us: 0,
                channel: 0,
                raw_value: 0,
                resolution_bits: 10,
                reference_mode: asv_protocol::AdcReferenceMode::Default,
                reference_mv: 5_000,
            }
        );
        for tick in 0..10_000 {
            let ProtocolEvent::AnalogSample {
                channel,
                raw_value,
                resolution_bits,
                ..
            } = mock_adc_event(tick, tick as u16)
            else {
                panic!("mock source must emit ADC events");
            };
            assert!(channel < 6);
            assert_eq!(resolution_bits, 10);
            assert!(raw_value <= 1_023);
        }
    }

    #[test]
    fn mock_pwm_source_uses_only_uno_hardware_pwm_pins() {
        assert_eq!(
            mock_pwm_event(0, 3),
            ProtocolEvent::PwmWrite {
                sequence: 3,
                board_timestamp_us: 0,
                pin: 3,
                duty_value: 0,
                resolution_bits: 8,
                output_mode: PwmOutputMode::ConstantLow,
                timer_number: 2,
                timer_channel: PwmTimerChannel::B,
                waveform_mode: PwmWaveformMode::PhaseCorrectPwm,
                output_polarity: PwmOutputPolarity::Disconnected,
                timer_clock_hz: 16_000_000,
                prescaler: 64,
                top: 255,
                compare_value: 0,
                counter_value: 0,
                control_a: 0x01,
                control_b: 0x04,
            }
        );
        for tick in 0..1_000 {
            let ProtocolEvent::PwmWrite {
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
                ..
            } = mock_pwm_event(tick, tick as u16)
            else {
                panic!("mock source must emit PWM events");
            };
            assert!([3, 5, 6, 9, 10, 11].contains(&pin));
            assert_eq!(resolution_bits, 8);
            assert!(duty_value <= 255);
            assert_eq!(timer_clock_hz, 16_000_000);
            assert_eq!(prescaler, 64);
            assert_eq!(top, 255);
            assert_eq!(compare_value, duty_value);
            assert!(counter_value <= top);
            let (expected_timer, expected_channel, expected_waveform) = match pin {
                3 => (2, PwmTimerChannel::B, PwmWaveformMode::PhaseCorrectPwm),
                5 => (0, PwmTimerChannel::B, PwmWaveformMode::FastPwm),
                6 => (0, PwmTimerChannel::A, PwmWaveformMode::FastPwm),
                9 => (1, PwmTimerChannel::A, PwmWaveformMode::PhaseCorrectPwm),
                10 => (1, PwmTimerChannel::B, PwmWaveformMode::PhaseCorrectPwm),
                11 => (2, PwmTimerChannel::A, PwmWaveformMode::PhaseCorrectPwm),
                _ => unreachable!(),
            };
            assert_eq!(timer_number, expected_timer);
            assert_eq!(timer_channel, expected_channel);
            assert_eq!(waveform_mode, expected_waveform);
            match output_mode {
                PwmOutputMode::ConstantLow | PwmOutputMode::ConstantHigh => {
                    assert_eq!(output_polarity, PwmOutputPolarity::Disconnected)
                }
                PwmOutputMode::HardwarePwm => {
                    assert_eq!(output_polarity, PwmOutputPolarity::NonInverting)
                }
            }
        }
    }
}
