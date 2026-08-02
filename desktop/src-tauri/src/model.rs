use asv_protocol::{AdcReferenceMode, GpioDirection, GpioLevel, GpioObservationSource};
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
