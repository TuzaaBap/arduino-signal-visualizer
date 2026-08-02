mod connection;
mod model;
mod validation;

use connection::ConnectionManager;
use validation::ValidationRecorder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ConnectionManager::default())
        .manage(ValidationRecorder::from_environment())
        .invoke_handler(tauri::generate_handler![
            connection::list_serial_ports,
            connection::connect_serial,
            connection::start_mock,
            connection::disconnect,
            validation::validation_start,
            validation::validation_acknowledge_gpio,
            validation::validation_acknowledge_adc,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Arduino Signal Visualizer");
}
