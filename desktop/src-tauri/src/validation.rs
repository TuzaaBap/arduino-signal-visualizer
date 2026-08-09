use std::{
    collections::BTreeMap,
    env, fs,
    path::PathBuf,
    sync::Mutex,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use asv_protocol::BoardDescriptor;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::{
    connection::{self, ConnectionManager},
    model::{
        AdcSample, ConnectionStatus, DiagnosticCategory, GpioBatch, GpioUpdate, ProtocolDiagnostic,
        PwmUpdate, UserSerialBatch,
    },
};

const REPORT_PATH_ENV: &str = "ASV_VALIDATION_REPORT";
const PORT_ENV: &str = "ASV_VALIDATION_PORT";
const RECONNECT_AFTER_ENV: &str = "ASV_VALIDATION_RECONNECT_AFTER_SECS";

pub struct ValidationRecorder {
    report_path: Option<PathBuf>,
    state: Mutex<ValidationSnapshot>,
    last_write: Mutex<Instant>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationSnapshot {
    schema_version: u8,
    application_version: &'static str,
    started_unix_ms: u128,
    last_update_unix_ms: u128,
    elapsed_ms: u128,
    board: Option<BoardDescriptor>,
    status_history: Vec<ConnectionStatus>,
    pins: BTreeMap<u8, PinValidation>,
    analog_channels: BTreeMap<u8, AdcValidation>,
    pwm_pins: BTreeMap<u8, PwmValidation>,
    received_gpio_updates: u64,
    received_adc_samples: u64,
    received_pwm_updates: u64,
    received_user_serial_bytes: u64,
    dropped_user_serial_bytes: u64,
    ui_acknowledgements: u64,
    ui_matches_backend: bool,
    ui_gpio_match_observed: bool,
    ui_adc_acknowledgements: u64,
    ui_adc_match_observed: bool,
    maximum_ui_adc_buffer_length: usize,
    ui_pwm_acknowledgements: u64,
    ui_pwm_match_observed: bool,
    maximum_ui_pwm_buffer_length: usize,
    diagnostics: Vec<ProtocolDiagnostic>,
    crc_failures: u64,
    dropped_packet_warnings: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct PinValidation {
    backend_update_count: u64,
    high_observations: u64,
    low_observations: u64,
    backend_latest: Option<GpioUpdate>,
    ui_latest: Option<GpioUpdate>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct AdcValidation {
    sample_count: u64,
    minimum_raw: Option<u16>,
    maximum_raw: Option<u16>,
    latest: Option<AdcSample>,
    ui_latest: Option<AdcSample>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct PwmValidation {
    update_count: u64,
    minimum_duty: Option<u16>,
    maximum_duty: Option<u16>,
    latest: Option<PwmUpdate>,
    ui_latest: Option<PwmUpdate>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdcUiChannelState {
    channel: u8,
    buffer_length: usize,
    latest: AdcSample,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PwmUiPinState {
    pin: u8,
    buffer_length: usize,
    latest: PwmUpdate,
}

impl ValidationRecorder {
    pub fn from_environment() -> Self {
        let report_path = if cfg!(feature = "hardware-validation") {
            env::var_os(REPORT_PATH_ENV).map(PathBuf::from)
        } else {
            None
        };
        let now = unix_time_ms();
        Self {
            report_path,
            state: Mutex::new(ValidationSnapshot {
                schema_version: 2,
                application_version: env!("CARGO_PKG_VERSION"),
                started_unix_ms: now,
                last_update_unix_ms: now,
                elapsed_ms: 0,
                board: None,
                status_history: Vec::new(),
                pins: BTreeMap::new(),
                analog_channels: BTreeMap::new(),
                pwm_pins: BTreeMap::new(),
                received_gpio_updates: 0,
                received_adc_samples: 0,
                received_pwm_updates: 0,
                received_user_serial_bytes: 0,
                dropped_user_serial_bytes: 0,
                ui_acknowledgements: 0,
                ui_matches_backend: true,
                ui_gpio_match_observed: false,
                ui_adc_acknowledgements: 0,
                ui_adc_match_observed: false,
                maximum_ui_adc_buffer_length: 0,
                ui_pwm_acknowledgements: 0,
                ui_pwm_match_observed: false,
                maximum_ui_pwm_buffer_length: 0,
                diagnostics: Vec::new(),
                crc_failures: 0,
                dropped_packet_warnings: 0,
            }),
            last_write: Mutex::new(Instant::now() - Duration::from_secs(2)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.report_path.is_some()
    }

    pub fn record_status(&self, status: ConnectionStatus) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| state.status_history.push(status));
        self.write_report(true);
    }

    pub fn record_board(&self, board: BoardDescriptor) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| state.board = Some(board));
        self.write_report(true);
    }

    pub fn record_batch(&self, batch: &GpioBatch) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.received_gpio_updates += batch.updates.len() as u64;
            for update in &batch.updates {
                let pin = state.pins.entry(update.pin).or_default();
                pin.backend_update_count += 1;
                match update.level {
                    asv_protocol::GpioLevel::Low => pin.low_observations += 1,
                    asv_protocol::GpioLevel::High => pin.high_observations += 1,
                }
                pin.backend_latest = Some(update.clone());
            }
        });
        self.write_report(false);
    }

    pub fn record_diagnostic(&self, diagnostic: ProtocolDiagnostic) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            if matches!(diagnostic.category, DiagnosticCategory::CorruptFrame)
                && diagnostic.message.to_ascii_lowercase().contains("crc")
            {
                state.crc_failures += 1;
            }
            if matches!(diagnostic.category, DiagnosticCategory::QueuePressure) {
                state.dropped_packet_warnings += 1;
            }
            state.diagnostics.push(diagnostic);
        });
        self.write_report(true);
    }

    pub fn record_adc_sample(&self, sample: AdcSample) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.received_adc_samples += 1;
            let channel = state.analog_channels.entry(sample.channel).or_default();
            channel.sample_count += 1;
            channel.minimum_raw = Some(
                channel
                    .minimum_raw
                    .map_or(sample.raw_value, |value| value.min(sample.raw_value)),
            );
            channel.maximum_raw = Some(
                channel
                    .maximum_raw
                    .map_or(sample.raw_value, |value| value.max(sample.raw_value)),
            );
            channel.latest = Some(sample);
        });
        self.write_report(false);
    }

    pub fn record_pwm_update(&self, update: PwmUpdate) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.received_pwm_updates += 1;
            let pin = state.pwm_pins.entry(update.pin).or_default();
            pin.update_count += 1;
            pin.minimum_duty = Some(
                pin.minimum_duty
                    .map_or(update.duty_value, |value| value.min(update.duty_value)),
            );
            pin.maximum_duty = Some(
                pin.maximum_duty
                    .map_or(update.duty_value, |value| value.max(update.duty_value)),
            );
            pin.latest = Some(update);
        });
        self.write_report(false);
    }

    pub fn record_user_serial(&self, batch: &UserSerialBatch) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.received_user_serial_bytes += batch.bytes.len() as u64;
            state.dropped_user_serial_bytes += batch.dropped_bytes;
        });
        self.write_report(false);
    }

    pub fn record_ui_state(&self, updates: Vec<GpioUpdate>) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.ui_acknowledgements += 1;
            for update in updates {
                let pin = update.pin;
                state.pins.entry(pin).or_default().ui_latest = Some(update);
            }
        });
        self.write_report(false);
    }

    pub fn record_adc_ui_state(&self, channels: Vec<AdcUiChannelState>) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.ui_adc_acknowledgements += 1;
            for channel in channels {
                state.maximum_ui_adc_buffer_length = state
                    .maximum_ui_adc_buffer_length
                    .max(channel.buffer_length);
                state
                    .analog_channels
                    .entry(channel.channel)
                    .or_default()
                    .ui_latest = Some(channel.latest);
            }
            if state.analog_channels.values().all(|channel| {
                channel
                    .latest
                    .as_ref()
                    .zip(channel.ui_latest.as_ref())
                    .is_some_and(|(backend, ui)| backend == ui)
            }) {
                state.ui_adc_match_observed = true;
            }
        });
        self.write_report(false);
    }

    pub fn record_pwm_ui_state(&self, pins: Vec<PwmUiPinState>) {
        if !self.enabled() {
            return;
        }
        self.with_state(|state| {
            state.ui_pwm_acknowledgements += 1;
            for pin in pins {
                state.maximum_ui_pwm_buffer_length =
                    state.maximum_ui_pwm_buffer_length.max(pin.buffer_length);
                state.pwm_pins.entry(pin.pin).or_default().ui_latest = Some(pin.latest);
            }
            if state.pwm_pins.values().all(|pin| {
                pin.latest
                    .as_ref()
                    .zip(pin.ui_latest.as_ref())
                    .is_some_and(|(backend, ui)| backend == ui)
            }) {
                state.ui_pwm_match_observed = true;
            }
        });
        self.write_report(false);
    }

    fn with_state(&self, update: impl FnOnce(&mut ValidationSnapshot)) {
        let mut state = self.state.lock().expect("validation mutex poisoned");
        update(&mut state);
        let now = unix_time_ms();
        state.last_update_unix_ms = now;
        state.elapsed_ms = now.saturating_sub(state.started_unix_ms);
        state.ui_matches_backend = state.pins.values().all(|pin| {
            pin.backend_latest
                .as_ref()
                .zip(pin.ui_latest.as_ref())
                .is_some_and(|(backend, ui)| {
                    backend.pin == ui.pin
                        && backend.direction == ui.direction
                        && backend.level == ui.level
                })
        });
        if state.ui_matches_backend && !state.pins.is_empty() {
            state.ui_gpio_match_observed = true;
        }
    }

    fn write_report(&self, force: bool) {
        let Some(path) = &self.report_path else {
            return;
        };
        let mut last_write = self.last_write.lock().expect("validation timer poisoned");
        if !force && last_write.elapsed() < Duration::from_secs(1) {
            return;
        }
        *last_write = Instant::now();
        let snapshot = self
            .state
            .lock()
            .expect("validation mutex poisoned")
            .clone();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_vec_pretty(&snapshot) {
            let _ = fs::write(path, json);
        }
    }
}

#[tauri::command]
pub fn validation_start(
    app: AppHandle,
    manager: State<'_, ConnectionManager>,
    recorder: State<'_, ValidationRecorder>,
) -> Result<bool, String> {
    if !recorder.enabled() {
        return Ok(false);
    }
    let port_name =
        env::var(PORT_ENV).map_err(|_| format!("{PORT_ENV} is required for validation"))?;
    connection::connect_serial_inner(port_name.clone(), 115_200, app.clone(), &manager)?;

    let reconnect_after = env::var(RECONNECT_AFTER_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(60);
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(reconnect_after));
        let manager = app.state::<ConnectionManager>();
        connection::disconnect_inner(app.clone(), &manager);
        std::thread::sleep(Duration::from_secs(2));
        let manager = app.state::<ConnectionManager>();
        let _ = connection::connect_serial_inner(port_name, 115_200, app.clone(), &manager);
    });
    Ok(true)
}

#[tauri::command]
pub fn validation_acknowledge_gpio(
    updates: Vec<GpioUpdate>,
    recorder: State<'_, ValidationRecorder>,
) {
    recorder.record_ui_state(updates);
}

#[tauri::command]
pub fn validation_acknowledge_adc(
    channels: Vec<AdcUiChannelState>,
    recorder: State<'_, ValidationRecorder>,
) {
    recorder.record_adc_ui_state(channels);
}

#[tauri::command]
pub fn validation_acknowledge_pwm(
    pins: Vec<PwmUiPinState>,
    recorder: State<'_, ValidationRecorder>,
) {
    recorder.record_pwm_ui_state(pins);
}

pub fn record_status(app: &AppHandle, status: ConnectionStatus) {
    app.state::<ValidationRecorder>().record_status(status);
}

pub fn record_board(app: &AppHandle, board: BoardDescriptor) {
    app.state::<ValidationRecorder>().record_board(board);
}

pub fn record_batch(app: &AppHandle, batch: &GpioBatch) {
    app.state::<ValidationRecorder>().record_batch(batch);
}

pub fn record_adc_sample(app: &AppHandle, sample: AdcSample) {
    app.state::<ValidationRecorder>().record_adc_sample(sample);
}

pub fn record_pwm_update(app: &AppHandle, update: PwmUpdate) {
    app.state::<ValidationRecorder>().record_pwm_update(update);
}

pub fn record_user_serial(app: &AppHandle, batch: &UserSerialBatch) {
    app.state::<ValidationRecorder>().record_user_serial(batch);
}

pub fn record_diagnostic(app: &AppHandle, diagnostic: ProtocolDiagnostic) {
    app.state::<ValidationRecorder>()
        .record_diagnostic(diagnostic);
}

fn unix_time_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
