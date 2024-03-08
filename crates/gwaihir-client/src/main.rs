#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::sensor_monitor_thread::{create_sensor_monitor_thread, MainToMonitorMessages};
pub use app::GwaihirApp;
use directories_next::ProjectDirs;
use egui::ViewportBuilder;
use flexi_logger::LoggerHandle;
use gwaihir_client_lib::APP_ID;
use log::info;
use sensors::lock_status_sensor::LockStatusSensorBuilder;
use std::path::PathBuf;

mod app;
mod networking;
pub mod notification;
mod periodic_repaint_thread;
mod persistence;
mod sensor_monitor_thread;
mod sensors;
pub mod triggers;
mod ui;
mod user_summaries;

// TODO: Re-introduce once the problems introduced by: https://github.com/emilk/egui/issues/3321, https://github.com/emilk/egui/pull/3831, https://github.com/emilk/egui/pull/3877, https://github.com/emilk/egui/pull/3985/files
// such as https://github.com/emilk/egui/issues/3655 and https://github.com/emilk/egui/issues/3902
// are fixed. After those changes, the main loop isn't called while the window is hidden.
// So until then, disable hide to tray functionality.
#[cfg(feature = "hide_to_try")]
mod tray_icon;

const ICON_BYTES: &[u8] = include_bytes!("../assets/eagle.png");

fn main() -> eframe::Result<()> {
    let logger = init_logging();
    info!("Starting Gwaihir");

    let mut native_options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_icon(eframe::icon_data::from_png_bytes(ICON_BYTES).unwrap()),
        ..Default::default()
    };
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
            Box::new(GwaihirApp::new(
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
    let project_dirs = project_dirs();
    let log_directory = if project_dirs.data_dir().ends_with("data") {
        project_dirs.data_dir().join("../logs")
    } else {
        project_dirs.data_dir().join("logs")
    };

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

pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", APP_ID).unwrap()
}

fn get_log_file_location(logger: &LoggerHandle) -> PathBuf {
    logger
        .existing_log_files()
        .unwrap()
        .into_iter()
        .find(|p| p.ends_with("gwaihir_rCURRENT.log"))
        .unwrap()
}
