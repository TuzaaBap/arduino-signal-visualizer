use asv_protocol::{
    AdcReferenceMode, GpioDirection, GpioLevel, GpioObservationSource, PwmOutputMode,
    PwmOutputPolarity, PwmTimerChannel, PwmWaveformMode,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialPortDescriptor {
    pub name: String,
    pub kind: String,
    pub usb_vendor_id: Option<u16>,
    pub usb_product_id: Option<u16>,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    pub phase: ConnectionPhase,
    pub mode: Option<ConnectionMode>,
    pub port_name: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionPhase {
    Disconnected,
    WaitingForHello,
    Connected,
    Error,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionMode {
    Serial,
    Mock,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpioUpdate {
    pub sequence: u16,
    pub board_timestamp_us: u32,
    pub pin: u8,
    pub direction: GpioDirection,
    pub level: GpioLevel,
    pub source: GpioObservationSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpioBatch {
    pub updates: Vec<GpioUpdate>,
    pub dropped_ui_events: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialActivityBatch {
    /// Bytes transmitted by the Uno USB bridge and received by the desktop.
    pub tx_bytes: u64,
    /// Bytes received by the Uno USB bridge from the desktop.
    pub rx_bytes: u64,
    /// Desktop activity-indicator hold time.
    pub pulse_duration_ms: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdcSample {
    pub sequence: u16,
    pub board_timestamp_us: u32,
    pub channel: u8,
    pub raw_value: u16,
    pub resolution_bits: u8,
    pub reference_mode: AdcReferenceMode,
    pub reference_mv: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdcBatch {
    pub samples: Vec<AdcSample>,
    pub coalesced_ui_samples: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PwmUpdate {
    pub sequence: u16,
    pub board_timestamp_us: u32,
    pub pin: u8,
    pub duty_value: u16,
    pub resolution_bits: u8,
    pub output_mode: PwmOutputMode,
    pub timer_number: u8,
    pub timer_channel: PwmTimerChannel,
    pub waveform_mode: PwmWaveformMode,
    pub output_polarity: PwmOutputPolarity,
    pub timer_clock_hz: u32,
    pub prescaler: u16,
    pub top: u16,
    pub compare_value: u16,
    pub counter_value: u16,
    pub control_a: u8,
    pub control_b: u8,
    pub period_ns: Option<u64>,
    pub high_time_ns: Option<u64>,
    pub low_time_ns: Option<u64>,
    pub frequency_millihz: Option<u64>,
    pub duty_ppm: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PwmBatch {
    pub updates: Vec<PwmUpdate>,
    pub coalesced_ui_updates: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolDiagnostic {
    pub category: DiagnosticCategory,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticCategory {
    CorruptFrame,
    MissingPackets,
    DuplicatePacket,
    OutOfOrderPacket,
    BoardReset,
    QueuePressure,
}
