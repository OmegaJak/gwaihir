use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::JoinHandle,
    time::Duration,
};

use egui::{epaint::ahash::HashSet, TextEdit, Widget};
use gwaihir_client_lib::{NetworkInterface, RemoteUpdate, UniqueUserId, UserStatus, APP_ID};

use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};

use crate::{
    sensor_monitor_thread::{MainToMonitorMessages, MonitorToMainMessages},
    sensors::{
        lock_status_sensor::{EventLoopRegisteredLockStatusSensorBuilder, LockStatusSensor},
        outputs::{sensor_output::SensorOutput, sensor_outputs::SensorOutputs},
    },
    tray_icon::{hide_to_tray, TrayIconData},
    widgets::auto_launch_checkbox::AutoLaunchCheckboxUiExtension,
};

#[derive(Serialize, Deserialize)]
pub struct Persistence {
    pub ignored_users: HashSet<UniqueUserId>,
}

impl Persistence {
    pub const STORAGE_KEY: &str = eframe::APP_KEY;
}

impl Default for Persistence {
    fn default() -> Self {
        Self {
            ignored_users: Default::default(),
        }
    }
}

pub struct GwaihirApp<N> {
    tray_icon_data: Option<TrayIconData>,

    sensor_monitor_thread_join_handle: Option<JoinHandle<()>>,

    tx_to_monitor_thread: Sender<MainToMonitorMessages>,
    rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
    current_status: HashMap<UniqueUserId, UserStatus<SensorOutputs>>,

    network: N,
    network_rx: Receiver<gwaihir_client_lib::RemoteUpdate<SensorOutputs>>,
    current_user_id: Option<UniqueUserId>,

    set_name_input: String,

    persistence: Persistence,
}

impl<N> GwaihirApp<N>
where
    N: NetworkInterface<SensorOutputs>,
{
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        sensor_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>>,
        tx_to_monitor_thread: Sender<MainToMonitorMessages>,
        rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
        sensor_monitor_thread_join_handle: JoinHandle<()>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let lock_status_sensor = init_lock_status_sensor(cc, sensor_builder);
        if let Some(sensor) = lock_status_sensor {
            tx_to_monitor_thread
                .send(MainToMonitorMessages::LockStatusSensorInitialized(sensor))
                .unwrap();
        }

        let persistence: Persistence = cc
            .storage
            .and_then(|storage| {
                return eframe::get_value(storage, Persistence::STORAGE_KEY);
            })
            .unwrap_or_default();

        let (network_tx, network_rx) = mpsc::channel();
        let ctx_clone = cc.egui_ctx.clone();
        let network: N = NetworkInterface::new(get_remote_update_callback(network_tx, ctx_clone));
        GwaihirApp {
            tray_icon_data: None,
            tx_to_monitor_thread,
            rx_from_monitor_thread,
            current_status: HashMap::new(),

            sensor_monitor_thread_join_handle: Some(sensor_monitor_thread_join_handle),

            network,
            network_rx,
            current_user_id: None,

            set_name_input: String::new(),

            persistence,
        }
    }

    fn get_filtered_sorted_user_status_list(
        &self,
    ) -> Vec<(UniqueUserId, UserStatus<SensorOutputs>)> {
        let mut user_status_list = self
            .current_status
            .iter()
            .filter(|(id, _)| self.subscribed_to_user(&id))
            .map(|(id, status)| (id.clone(), status.clone()))
            .collect::<Vec<_>>();

        // Sort to ensure current user is on top
        user_status_list.sort_by(|(id_a, _), (id_b, _)| {
            if self
                .current_user_id
                .as_ref()
                .is_some_and(|own_id| own_id == id_a)
            {
                Ordering::Less
            } else if self
                .current_user_id
                .as_ref()
                .is_some_and(|own_id| own_id == id_b)
            {
                Ordering::Equal
            } else {
                id_a.cmp(id_b)
            }
        });

        user_status_list
    }
}

impl<N> eframe::App for GwaihirApp<N>
where
    N: NetworkInterface<SensorOutputs>,
{
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Persistence::STORAGE_KEY, &self.persistence);
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Some(join_handle) = self.sensor_monitor_thread_join_handle.take() {
            self.tx_to_monitor_thread
                .send(MainToMonitorMessages::Shutdown)
                .unwrap();
            join_handle.join().ok();
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.rx_from_monitor_thread.try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                panic!("The background thread unexpected disconnected!");
            }
            Ok(MonitorToMainMessages::UpdatedSensorOutputs(sensor_outputs)) => {
                // println!("Publishing update: {:#?}", &sensor_outputs);
                self.network.publish_update(sensor_outputs);
            }
        }

        while let Ok(update) = self.network_rx.try_recv() {
            match update {
                RemoteUpdate::UserStatusUpdated(status) => {
                    if self.subscribed_to_user(&status.user_id) {
                        // println!("Got user update from DB: {:#?}", &status);
                        self.current_status.insert(status.user_id.clone(), status);
                    }
                }
            };
        }

        if self.current_user_id.is_none() {
            self.current_user_id = self.network.get_current_user_id();
        }

        if let Some(icon_data) = self.tray_icon_data.take() {
            // println!("Checking tray");
            self.tray_icon_data = crate::tray_icon::handle_events(frame, icon_data);
            ctx.request_repaint_after(Duration::from_millis(100000000));
            return;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    ui.auto_launch_checkbox(APP_ID.to_string(), None);

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
            let user_status_list = self.get_filtered_sorted_user_status_list();
            for (id, status) in user_status_list.iter() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    status.sensor_outputs.show_first(
                        |o| matches!(o, SensorOutput::OnlineStatus(_)),
                        ui,
                        id,
                    );
                    ui.heading(status.display_name());
                    if let Some(current_user_id) = &self.current_user_id {
                        if id == current_user_id {
                            let text_edit_response = TextEdit::singleline(&mut self.set_name_input)
                                .desired_width(100.0)
                                .ui(ui);
                            if ui.button("Set Username").clicked()
                                || (text_edit_response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                            {
                                self.network.set_username(self.set_name_input.clone());
                                self.set_name_input = String::new();
                            }
                        } else if ui.button("x").clicked() {
                            self.persistence.ignored_users.insert((*id).clone().into());
                        }
                    }
                });

                status.sensor_outputs.show_first(
                    |o| matches!(o, SensorOutput::LockStatus(_)),
                    ui,
                    id,
                );

                status.sensor_outputs.show_first(
                    |o| matches!(o, SensorOutput::MicrophoneUsage(_)),
                    ui,
                    id,
                );
            }

            egui::warn_if_debug_build(ui);
        });
    }
}

impl<N> GwaihirApp<N> {
    fn subscribed_to_user(&self, user_id: &UniqueUserId) -> bool {
        !self.persistence.ignored_users.contains(user_id)
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

fn get_remote_update_callback(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    ctx_clone: egui::Context,
) -> impl Fn(RemoteUpdate<SensorOutputs>) + Clone {
    move |update| {
        network_tx.send(update).unwrap();
        ctx_clone.request_repaint();
    }
}
