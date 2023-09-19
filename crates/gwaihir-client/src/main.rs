#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;

use crate::sensor_monitor_thread::{create_sensor_monitor_thread, MainToMonitorMessages};
pub use app::GwaihirApp;
use directories_next::ProjectDirs;
use flexi_logger::LoggerHandle;
use gwaihir_client_lib::APP_ID;
use log::info;
use networking_spacetimedb::SpacetimeDBInterface;
use sensors::lock_status_sensor::LockStatusSensorBuilder;

mod app;
mod networking;
mod periodic_repaint_thread;
mod sensor_monitor_thread;
mod sensors;
mod tray_icon;
mod ui;

fn main() -> eframe::Result<()> {
    let logger = init_logging();
    info!("Starting Gwaihir");

    let mut native_options = eframe::NativeOptions::default();
    native_options.app_id = Some(APP_ID.to_string());
    let registered_builder =
        LockStatusSensorBuilder::new().set_event_loop_builder(&mut native_options);
    let (monitor_handle, tx_to_monitor, rx_from_monitor) = create_sensor_monitor_thread();
    let log_file_location = get_log_file_location(&logger);
    eframe::run_native(
        "Gwaihir",
        native_options,
        Box::new(move |cc| {
            let ctx_clone = cc.egui_ctx.clone();
            tx_to_monitor
                .send(MainToMonitorMessages::SetEguiContext(ctx_clone))
                .unwrap();
            Box::new(GwaihirApp::new::<SpacetimeDBInterface>(
                cc,
                registered_builder,
                tx_to_monitor,
                rx_from_monitor,
                monitor_handle,
                log_file_location,
            ))
        }),
    )?;

    info!("Gwaihir closing nominally");
    Ok(())
}

fn init_logging() -> LoggerHandle {
    let log_directory = ProjectDirs::from("", "", APP_ID)
        .unwrap()
        .data_dir()
        .join("..")
        .join("logs");

    let handle = flexi_logger::Logger::try_with_env_or_str("info")
        .unwrap()
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stdout(flexi_logger::colored_detailed_format)
        .log_to_file(flexi_logger::FileSpec::default().directory(log_directory))
        .duplicate_to_stdout(flexi_logger::Duplicate::All)
        .append()
        .rotate(
            flexi_logger::Criterion::Age(flexi_logger::Age::Hour),
            flexi_logger::Naming::Timestamps,
            flexi_logger::Cleanup::KeepLogFiles(3),
        )
        .cleanup_in_background_thread(true)
        .start()
        .unwrap();
    log_panics::init();

    handle
}

fn get_log_file_location(logger: &LoggerHandle) -> PathBuf {
    logger
        .existing_log_files()
        .unwrap()
        .into_iter()
        .find(|p| p.ends_with("gwaihir_rCURRENT.log"))
        .unwrap()
}
