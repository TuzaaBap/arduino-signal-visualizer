#[cfg(desktop)]
mod app_updates;
mod connection;
mod model;
mod validation;

use connection::ConnectionManager;
use validation::ValidationRecorder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_updates::UpdateManager::default());

    builder
        .manage(ConnectionManager::default())
        .manage(ValidationRecorder::from_environment())
        .invoke_handler(tauri::generate_handler![
            connection::list_serial_ports,
            connection::connect_serial,
            connection::start_mock,
            connection::disconnect,
            connection::write_user_serial,
            validation::validation_start,
            validation::validation_acknowledge_gpio,
            validation::validation_acknowledge_adc,
            validation::validation_acknowledge_pwm,
            #[cfg(desktop)]
            app_updates::check_for_update,
            #[cfg(desktop)]
            app_updates::install_update,
            #[cfg(desktop)]
            app_updates::dismiss_update,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Arduino Signal Visualizer");
}
