use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::Duration,
};

use gwaihir_client_lib::{NetworkInterface, RemoteUpdate, SensorData};
use networking_spacetimedb::SpacetimeDBInterface;
use raw_window_handle::HasRawWindowHandle;

use crate::{
    lock_status_sensor::{EventLoopRegisteredLockStatusSensorBuilder, LockStatusSensor},
    sensor_monitor_thread::{MainToMonitorMessages, MonitorToMainMessages},
    tray_icon::{hide_to_tray, TrayIconData},
    ui_extension_methods::UIExtensionMethods,
};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// #[derive(serde::Serialize)]
// #[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp<N> {
    // Example stuff:
    label: String,

    // this how you opt-out of serialization of a member
    // #[serde(skip)]
    value: f32,

    // #[serde(skip)]
    tray_icon_data: Option<TrayIconData>,

    // #[serde(skip)]
    tx_to_monitor_thread: Sender<MainToMonitorMessages>,
    rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
    last_sensor_data: SensorData,

    network: N,
}

// impl Default for TemplateApp {
//     fn default() -> Self {
//         Self {
//             // Example stuff:
//             label: "Hello World!".to_owned(),
//             value: 2.7,
//             tray_icon_data: None,
//             lock_status_sensor: None,
//             microphone_usage_sensor: MicrophoneUsageSensor::new(),
//         }
//     }
// }

impl<N> TemplateApp<N>
where
    N: NetworkInterface,
{
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        sensor_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>>,
        tx_to_monitor_thread: Sender<MainToMonitorMessages>,
        rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let lock_status_sensor = init_lock_status_sensor(cc, sensor_builder);
        if let Some(sensor) = lock_status_sensor {
            tx_to_monitor_thread
                .send(MainToMonitorMessages::LockStatusSensorInitialized(sensor))
                .unwrap();
        }

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // let mut app: TemplateApp = if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // };

        TemplateApp {
            label: "Hello World!".to_owned(),
            value: 2.7,
            tray_icon_data: None,
            tx_to_monitor_thread,
            rx_from_monitor_thread,
            last_sensor_data: SensorData::default(),

            network: NetworkInterface::new(),
        }
    }
}

impl<N> eframe::App for TemplateApp<N>
where
    N: NetworkInterface,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.rx_from_monitor_thread.try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                panic!("The background thread unexpected disconnected!");
            }
            Ok(MonitorToMainMessages::UpdatedSensorData(sensor_data)) => {
                self.network.publish_status_update(sensor_data);
            }
        }

        for remote_update in self.network.receive_updates().into_iter() {
            match remote_update {
                RemoteUpdate::UserStatusUpdated(user_name, sensor_data, update_time) => {
                    println!("Sensor update for {} at {}", user_name, update_time);
                    self.last_sensor_data = sensor_data;
                }
            }
        }

        if let Some(icon_data) = self.tray_icon_data.take() {
            // println!("Checking tray");
            self.tray_icon_data = crate::tray_icon::handle_events(frame, icon_data);
            ctx.request_repaint_after(Duration::from_millis(100000000));
            return;
        }

        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Hide to tray").clicked() {
                        let tray_icon_data = hide_to_tray(frame);
                        self.tray_icon_data = Some(tray_icon_data);
                        ui.close_menu();
                    }

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Side Panel");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("powered by ");
                    ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                    ui.label(" and ");
                    ui.hyperlink_to(
                        "eframe",
                        "https://github.com/emilk/egui/tree/master/crates/eframe",
                    );
                    ui.label(".");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("Sensor Data");
            ui.collapsing_default_open("Lock/Unlocks", |ui| {
                ui.label(format!("Times Locked: {}", self.last_sensor_data.num_locks));
                ui.label(format!(
                    "Times Unlocked: {}",
                    self.last_sensor_data.num_unlocks
                ));
            });
            ui.collapsing_default_open("Microphone Usage", |ui| {
                ui.label(format!(
                    "{} app(s) currently listening to the microphone",
                    self.last_sensor_data.microphone_usage.len()
                ));
                for usage in self.last_sensor_data.microphone_usage.iter() {
                    let pretty_name = usage.app_name.replace("#", "\\");
                    ui.label(pretty_name);
                }
            });

            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}

fn init_lock_status_sensor(
    cc: &eframe::CreationContext<'_>,
    sensor_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>>,
) -> Option<LockStatusSensor> {
    match cc.raw_window_handle() {
        raw_window_handle::RawWindowHandle::Win32(handle) => {
            match sensor_builder.take().expect("The lock status sensor builder should be ready when we initialize the Template App").register_os_hook(handle) {
                Ok(builder) => Some(builder.build()),
                Err(err) => {
                    eprintln!("{:#?}", err);
                    None
                }
            }
        }
        _ => todo!(),
    }
}
