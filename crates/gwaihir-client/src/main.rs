#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::sensor_monitor_thread::{create_sensor_monitor_thread, MainToMonitorMessages};
pub use app::GwaihirApp;
use gwaihir_client_lib::APP_ID;
use networking_spacetimedb::SpacetimeDBInterface;
use sensors::lock_status_sensor::LockStatusSensorBuilder;

mod app;
mod sensor_monitor_thread;
mod sensors;
mod tray_icon;
mod ui_extension_methods;
mod widgets;

fn main() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let mut native_options = eframe::NativeOptions::default();
    native_options.app_id = Some(APP_ID.to_string());
    let registered_builder =
        LockStatusSensorBuilder::new().set_event_loop_builder(&mut native_options);
    let (monitor_handle, tx_to_monitor, rx_from_monitor) = create_sensor_monitor_thread();
    eframe::run_native(
        "Gwaihir",
        native_options,
        Box::new(move |cc| {
            let ctx_clone = cc.egui_ctx.clone();
            tx_to_monitor
                .send(MainToMonitorMessages::SetEguiContext(ctx_clone))
                .unwrap();
            Box::new(GwaihirApp::<SpacetimeDBInterface>::new(
                cc,
                registered_builder,
                tx_to_monitor,
                rx_from_monitor,
            ))
        }),
    )?;
    monitor_handle.join().unwrap();
    Ok(())
}
