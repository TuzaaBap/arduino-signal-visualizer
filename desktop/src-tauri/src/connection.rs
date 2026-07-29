use std::{
    collections::BTreeMap,
    io::{ErrorKind, Read},
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
    ProtocolEvent, SequenceObservation, SequenceTracker, StreamDecoder,
};
use serialport::{SerialPort, SerialPortType};
use tauri::{AppHandle, Emitter, State};

use crate::model::{
    ConnectionMode, ConnectionPhase, ConnectionStatus, DiagnosticCategory, GpioBatch, GpioUpdate,
    ProtocolDiagnostic, SerialPortDescriptor,
};
use crate::validation;

const EVENT_CONNECTION_STATUS: &str = "asv://connection-status";
const EVENT_BOARD_INFO: &str = "asv://board-info";
const EVENT_GPIO_BATCH: &str = "asv://gpio-batch";
const EVENT_DIAGNOSTIC: &str = "asv://protocol-diagnostic";
const UI_QUEUE_CAPACITY: usize = 256;
const UI_FLUSH_INTERVAL: Duration = Duration::from_millis(33);
const HELLO_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Default)]
pub struct ConnectionManager {
    active: Mutex<Option<ActiveConnection>>,
}

struct ActiveConnection {
    stop: Arc<AtomicBool>,
    handles: Vec<JoinHandle<()>>,
}

enum SourceMessage {
    Event(ProtocolEvent),
    Diagnostic(ProtocolDiagnostic),
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
            let detail = format!("Could not open {port_name}: {error}");
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

    let active = start_serial_workers(app, port, port_name);
    manager.replace(active);
    Ok(())
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
) -> ActiveConnection {
    let stop = Arc::new(AtomicBool::new(false));
    let dropped = Arc::new(AtomicU64::new(0));
    let (sender, receiver) = mpsc::sync_channel(UI_QUEUE_CAPACITY);

    let reader_stop = Arc::clone(&stop);
    let reader_dropped = Arc::clone(&dropped);
    let reader_port_name = port_name.clone();
    let reader_app = app.clone();
    let reader = thread::spawn(move || {
        let mut bytes = [0_u8; 256];
        let mut decoder = StreamDecoder::resynchronizing();
        let mut tracker = SequenceTracker::default();
        let mut handshake_complete = false;

        while !reader_stop.load(Ordering::Relaxed) {
            match port.read(&mut bytes) {
                Ok(count) if count > 0 => {
                    for result in decoder.push(&bytes[..count]) {
                        match result {
                            Ok(packet)
                                if handshake_complete
                                    || packet.packet_type == PacketType::BoardHello =>
                            {
                                handshake_complete = true;
                                handle_packet(&sender, &reader_dropped, &mut tracker, packet);
                            }
                            Ok(_) => {}
                            Err(error) if handshake_complete => try_send(
                                &sender,
                                &reader_dropped,
                                SourceMessage::Diagnostic(ProtocolDiagnostic {
                                    category: DiagnosticCategory::CorruptFrame,
                                    message: error.to_string(),
                                }),
                            ),
                            Err(_) => {}
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
            dropped,
            ConnectionMode::Serial,
            Some(port_name),
        );
    });

    ActiveConnection {
        stop,
        handles: vec![reader, delivery],
    }
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
                minor: 1,
                patch: 0,
            },
            capabilities: 1,
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
            dropped,
            ConnectionMode::Mock,
            None,
        );
    });

    ActiveConnection {
        stop,
        handles: vec![producer, delivery],
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

fn handle_packet(
    sender: &SyncSender<SourceMessage>,
    dropped: &AtomicU64,
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

fn try_send(sender: &SyncSender<SourceMessage>, dropped: &AtomicU64, message: SourceMessage) {
    match sender.try_send(message) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            dropped.fetch_add(1, Ordering::Relaxed);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

fn deliver_events(
    app: AppHandle,
    receiver: Receiver<SourceMessage>,
    stop: Arc<AtomicBool>,
    dropped: Arc<AtomicU64>,
    mode: ConnectionMode,
    port_name: Option<String>,
) {
    let started = Instant::now();
    let mut hello_received = false;
    let mut timeout_reported = false;
    let mut latest_gpio = BTreeMap::<u8, GpioUpdate>::new();
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
            next_flush = Instant::now() + UI_FLUSH_INTERVAL;
        }
        thread::sleep(Duration::from_millis(4));
    }
    flush_gpio(&app, &mut latest_gpio, &dropped);
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
}
