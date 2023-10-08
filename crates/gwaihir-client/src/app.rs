use chrono_humanize::HumanTime;
use egui::{epaint::ahash::HashSet, Color32, RichText, TextEdit, Widget};
use gwaihir_client_lib::{
    chrono::Local, NetworkInterface, RemoteUpdate, UniqueUserId, UserStatus, APP_ID,
};
use log::{debug, info, warn};
use log_err::LogErrResult;
use networking_spacetimedb::{SpacetimeDBCreationParameters, SpacetimeDBInterface};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread::JoinHandle,
    time::Duration,
};

use crate::{
    networking::network_manager::NetworkManager,
    periodic_repaint_thread::create_periodic_repaint_thread,
    sensor_monitor_thread::{MainToMonitorMessages, MonitorToMainMessages},
    sensors::{
        lock_status_sensor::{init_lock_status_sensor, EventLoopRegisteredLockStatusSensorBuilder},
        outputs::{sensor_output::SensorOutput, sensor_outputs::SensorOutputs},
    },
    tray_icon::{hide_to_tray, TrayIconData},
    ui::{
        network_window::NetworkWindow, time_formatting::nicely_formatted_datetime,
        transmission_spy::TransmissionSpy,
        widgets::auto_launch_checkbox::AutoLaunchCheckboxUiExtension,
    },
};

#[derive(Serialize, Deserialize)]
pub struct Persistence {
    pub ignored_users: HashSet<UniqueUserId>,
    pub spacetimedb_db_name: String,
}

impl Persistence {
    pub const STORAGE_KEY: &str = eframe::APP_KEY;
}

impl Default for Persistence {
    fn default() -> Self {
        Self {
            ignored_users: Default::default(),
            spacetimedb_db_name: "gwaihir-test".to_string(),
        }
    }
}

pub struct GwaihirApp {
    tray_icon_data: Option<TrayIconData>,

    sensor_monitor_thread_join_handle: Option<JoinHandle<()>>,
    tx_to_monitor_thread: Sender<MainToMonitorMessages>,
    rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
    current_status: HashMap<UniqueUserId, UserStatus<SensorOutputs>>,

    _periodic_repaint_thread_join_handle: JoinHandle<()>,

    network: NetworkManager,
    current_user_id: Option<UniqueUserId>,

    set_name_input: String,

    persistence: Persistence,
    log_file_location: PathBuf,

    network_window: NetworkWindow,
    transmission_spy: TransmissionSpy,
}

impl GwaihirApp {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        sensor_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>>,
        tx_to_monitor_thread: Sender<MainToMonitorMessages>,
        rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
        sensor_monitor_thread_join_handle: JoinHandle<()>,
        log_file_location: PathBuf,
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
            .and_then(|storage| eframe::get_value(storage, Persistence::STORAGE_KEY))
            .unwrap_or_default();

        let periodic_repaint_thread_join_handle =
            create_periodic_repaint_thread(cc.egui_ctx.clone(), Duration::from_secs(10));

        let creation_params = SpacetimeDBCreationParameters {
            db_name: persistence.spacetimedb_db_name.clone(),
        };
        let network =
            NetworkManager::new::<SpacetimeDBInterface, _>(cc.egui_ctx.clone(), creation_params);
        GwaihirApp {
            tray_icon_data: None,
            tx_to_monitor_thread,
            rx_from_monitor_thread,
            current_status: HashMap::new(),

            network_window: NetworkWindow::new(&network),
            transmission_spy: TransmissionSpy::new(),

            sensor_monitor_thread_join_handle: Some(sensor_monitor_thread_join_handle),
            network,
            current_user_id: None,

            _periodic_repaint_thread_join_handle: periodic_repaint_thread_join_handle,

            set_name_input: String::new(),

            persistence,
            log_file_location,
        }
    }

    fn get_filtered_sorted_user_status_list(
        &self,
    ) -> Vec<(UniqueUserId, UserStatus<SensorOutputs>)> {
        let mut user_status_list = self
            .current_status
            .iter()
            .filter(|(id, _)| self.subscribed_to_user(id))
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

    fn gracefully_shutdown_sensor_monitor_thread(&mut self) {
        if let Some(join_handle) = self.sensor_monitor_thread_join_handle.take() {
            info!("Sending shutdown message to monitor thread");
            self.tx_to_monitor_thread
                .send(MainToMonitorMessages::Shutdown)
                .unwrap();
            let mut attempts: u8 = 0;
            const SLEEP_DURATION: Duration = Duration::from_millis(20);
            const MAX_ATTEMPTS: u8 = 10;
            while !join_handle.is_finished() && attempts < MAX_ATTEMPTS {
                std::thread::sleep(SLEEP_DURATION);
                attempts += 1;
            }

            if join_handle.is_finished() {
                join_handle.join().ok();
                info!("Sensor monitor thread successfully shut down (after sleeping for {}ms {} time(s))", SLEEP_DURATION.as_millis(), attempts)
            } else {
                warn!(
                    "Sensor monitor thread was still not finished after {} attempts ({}ms between each)! Terminating application without joining thread...",
                    attempts,
                    SLEEP_DURATION.as_millis()
                );
            }
        }
    }
}

impl eframe::App for GwaihirApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Persistence::STORAGE_KEY, &self.persistence);
    }

    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.gracefully_shutdown_sensor_monitor_thread();
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
                debug!("Publishing update: {:#?}", &sensor_outputs);
                self.transmission_spy.record_update(sensor_outputs.clone());
                self.network.publish_update(sensor_outputs);
            }
        }

        self.network.try_reconnect_if_needed();
        while let Ok(update) = self.network.try_recv() {
            match update {
                RemoteUpdate::UserStatusUpdated(status) => {
                    if self.subscribed_to_user(&status.user_id) {
                        debug!("Got user update from DB: {:#?}", &status);
                        self.current_status.insert(status.user_id.clone(), status);
                    }
                }
            };
        }

        if self.current_user_id.is_none() {
            self.current_user_id = self.network.get_current_user_id();
        }

        if let Some(icon_data) = self.tray_icon_data.take() {
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

                    if ui.button("Open log").clicked() {
                        opener::open(self.log_file_location.clone())
                            .log_expect("Failed to open file using default OS handler");
                    }

                    if ui.button("Manage Network").clicked() {
                        self.network_window.set_shown(true);
                        ui.close_menu();
                    }

                    if ui.button("View Sent Data").clicked() {
                        self.transmission_spy.set_shown(true);
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
            if self.network.is_offline() {
                ui.label(RichText::new("⚠⚠ OFFLINE ⚠⚠").heading().color(Color32::RED));
            }

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
                    ui.label(RichText::new(format!(
                        " {} ",
                        HumanTime::from(status.last_update)
                    )))
                    .on_hover_text_at_pointer(format!(
                        "Last updated: {}",
                        nicely_formatted_datetime(status.last_update.with_timezone(&Local),)
                    ));

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
                            self.persistence.ignored_users.insert((*id).clone());
                        }
                    }
                });

                status.sensor_outputs.show_first(
                    |o| matches!(o, SensorOutput::LockStatus(_)),
                    ui,
                    id,
                );

                status.sensor_outputs.show_first(
                    |o| matches!(o, SensorOutput::SummarizedWindowActivity(_)),
                    ui,
                    id,
                );

                status.sensor_outputs.show_first(
                    |o| matches!(o, SensorOutput::KeyboardMouseActivity(_)),
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

        self.network_window
            .show(ctx, &mut self.network, &mut self.persistence);
        self.transmission_spy.show(ctx);
    }
}

impl GwaihirApp {
    fn subscribed_to_user(&self, user_id: &UniqueUserId) -> bool {
        !self.persistence.ignored_users.contains(user_id)
    }
}
