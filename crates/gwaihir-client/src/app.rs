use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    time::Duration,
};

use egui::CollapsingHeader;
use gwaihir_client_lib::{
    chrono::{DateTime, Utc},
    NetworkInterface, RemoteUpdate, SensorData, UniqueUserId,
};
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
    current_status: HashMap<UniqueUserId, UserStatus>,

    network: N,
    network_rx: Receiver<RemoteUpdate>,
    current_user_id: UniqueUserId,

    set_name_input: String,
}

pub struct UserStatus {
    pub display_name: String,
    pub is_online: bool,
    pub sensor_data: SensorData,
    pub last_update: DateTime<Utc>,
}

impl UserStatus {
    fn update(&mut self, remote_update: &RemoteUpdate) {
        match remote_update {
            RemoteUpdate::UserStatusUpdated(
                user_id,
                username,
                online,
                sensor_data,
                update_time,
            ) => {
                self.display_name = if username.as_ref().is_empty() {
                    user_id.clone().into()
                } else {
                    username.clone().into()
                };
                self.sensor_data = sensor_data.clone();
                self.last_update = update_time.clone();
            }
        }
    }
}

impl From<RemoteUpdate> for UserStatus {
    fn from(update: RemoteUpdate) -> Self {
        match update {
            RemoteUpdate::UserStatusUpdated(_, _, _, _, _) => {
                let mut status = UserStatus::default();
                status.update(&update);
                status
            }
        }
    }
}

impl Default for UserStatus {
    fn default() -> Self {
        Self {
            is_online: false,
            display_name: Default::default(),
            sensor_data: Default::default(),
            last_update: Default::default(),
        }
    }
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

        let (network_tx, network_rx) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();
        let network: N = NetworkInterface::new(get_remote_update_callback(network_tx, ctx_clone));
        let current_user_id = network.get_current_user_id();
        TemplateApp {
            label: "Hello World!".to_owned(),
            value: 2.7,
            tray_icon_data: None,
            tx_to_monitor_thread,
            rx_from_monitor_thread,
            current_status: HashMap::new(),

            network,
            network_rx,
            current_user_id,

            set_name_input: String::new(),
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

        while let Ok(update) = self.network_rx.try_recv() {
            let user_id = match &update {
                RemoteUpdate::UserStatusUpdated(user_id, _, _, _, _) => user_id,
            };

            if subscribed_to_user(user_id) {
                self.current_status
                    .entry(user_id.clone())
                    .and_modify(|status| status.update(&update))
                    .or_insert(update.into());
            }
        }
        // println!("Sensor update for {} at {}", descriptor, update_time);

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

                ui.separator();
                ui.label(format!("Frame: {}", ctx.frame_nr()));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            for (id, status) in &self.current_status {
                ui.horizontal(|ui| {
                    ui.heading(status.display_name.clone());
                    if id == &self.current_user_id {
                        ui.text_edit_singleline(&mut self.set_name_input);
                        if ui.button("Set Username").clicked() {
                            self.network.set_username(self.set_name_input.clone());
                        }
                    }
                });
                CollapsingHeader::new("Locks/Unlocks")
                    .default_open(true)
                    .id_source(format!("{}_locks", id.as_ref()))
                    .show(ui, |ui| {
                        ui.label(format!("Times Locked: {}", status.sensor_data.num_locks));
                        ui.label(format!(
                            "Times Unlocked: {}",
                            status.sensor_data.num_unlocks
                        ));
                    });
                CollapsingHeader::new("Microphone Usage")
                    .default_open(true)
                    .id_source(format!("{}_mic", id.as_ref()))
                    .show(ui, |ui| {
                        ui.label(format!(
                            "{} app(s) currently listening to the microphone",
                            status.sensor_data.microphone_usage.len()
                        ));
                        for usage in status.sensor_data.microphone_usage.iter() {
                            let pretty_name = usage.app_name.replace("#", "\\");
                            ui.label(pretty_name);
                        }
                    });
            }

            egui::warn_if_debug_build(ui);
        });
    }
}

fn subscribed_to_user(user_id: &gwaihir_client_lib::UniqueUserId) -> bool {
    true
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

fn get_remote_update_callback(
    network_tx: Sender<RemoteUpdate>,
    ctx_clone: egui::Context,
) -> impl Fn(RemoteUpdate) {
    move |update| {
        network_tx.send(update).unwrap();
        ctx_clone.request_repaint();
    }
}
