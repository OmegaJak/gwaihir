use crate::{
    networking::network_manager::NetworkManager,
    notification::{NotificationDispatch, OSNotificationDispatch},
    periodic_repaint_thread::create_periodic_repaint_thread,
    persistence::{Persistence, PersistenceV1, VersionedPersistence},
    project_dirs,
    sensor_monitor_thread::{MainToMonitorMessages, MonitorToMainMessages},
    sensors::{
        lock_status_sensor::{init_lock_status_sensor, EventLoopRegisteredLockStatusSensorBuilder},
        outputs::{
            online_status::OnlineStatus,
            sensor_output::{SensorOutput, SensorWidget},
            sensor_outputs::SensorOutputs,
        },
    },
    tray_icon::{hide_to_tray, TrayIconData},
    triggers::{ui::TriggersWindow, BehaviorOnTrigger, TriggerManager, Update},
    ui::{
        add_fake_user_window::AddFakeUserWindow,
        network_window::NetworkWindow,
        raw_data_window::{RawDataWindow, TimestampedData},
        time_formatting::nicely_formatted_datetime,
        ui_extension_methods::UIExtensionMethods,
        widgets::auto_launch_checkbox::AutoLaunchCheckboxUiExtension,
    },
    user_summaries::UserSummaries,
};
use chrono_humanize::HumanTime;
use egui::{Color32, RichText, ScrollArea};
use gwaihir_client_lib::{
    chrono::{Local, Utc},
    NetworkInterface, RemoteUpdate, UniqueUserId, UserStatus, APP_ID,
};
use log::{debug, info, warn};
use log_err::LogErrResult;
use networking_spacetimedb::{SpacetimeDBCreationParameters, SpacetimeDBInterface};
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ffi::OsStr,
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{Receiver, Sender, TryRecvError},
    thread::JoinHandle,
    time::Duration,
};

pub struct GwaihirApp {
    tray_icon_data: Option<TrayIconData>,

    sensor_monitor_thread_join_handle: Option<JoinHandle<()>>,
    tx_to_monitor_thread: Sender<MainToMonitorMessages>,
    rx_from_monitor_thread: Receiver<MonitorToMainMessages>,
    current_status: HashMap<UniqueUserId, UserStatus<SensorOutputs>>,
    user_summaries: UserSummaries,

    _periodic_repaint_thread_join_handle: JoinHandle<()>,

    network: NetworkManager,
    current_user_id: Option<UniqueUserId>,

    persistence: Persistence,
    log_file_location: PathBuf,

    network_window: NetworkWindow,
    transmission_spy: RawDataWindow,
    received_data_viewer: RawDataWindow,
    add_fake_user_window: AddFakeUserWindow,
    triggers_window: TriggersWindow,
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

        let persistence = load_and_migrate_persistence(cc, &log_file_location);

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
            user_summaries: UserSummaries::new(),

            network_window: NetworkWindow::new(&network),
            transmission_spy: RawDataWindow::new("Last Sent Data".to_string()),
            received_data_viewer: RawDataWindow::new("Raw Data".to_string()),

            sensor_monitor_thread_join_handle: Some(sensor_monitor_thread_join_handle),
            network,
            current_user_id: None,

            _periodic_repaint_thread_join_handle: periodic_repaint_thread_join_handle,

            persistence,
            log_file_location,

            add_fake_user_window: AddFakeUserWindow::new(),
            triggers_window: TriggersWindow::new(),
        }
    }

    fn get_filtered_sorted_user_statuses(&self) -> Vec<(UniqueUserId, UserStatus<SensorOutputs>)> {
        let mut user_statuses = self
            .current_status
            .iter()
            .filter(|(id, _)| self.subscribed_to_user(id))
            .map(|(id, status)| (id.clone(), status.clone()))
            .collect::<Vec<_>>();

        // Sort to ensure current user is on top
        user_statuses.sort_by(|(id_a, _), (id_b, _)| {
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

        user_statuses
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

    fn show_user_context_menu(
        &mut self,
        target_user_id: &UniqueUserId,
        ui: &mut egui::Ui,
        user_status: &UserStatus<SensorOutputs>,
    ) {
        match &self.current_user_id {
            Some(current_user_id) => {
                if target_user_id == current_user_id {
                    ui.name_input("Set Username", "set_username_input", |name| {
                        self.network.set_username(name)
                    });
                }

                if ui.button("Ignore").clicked() {
                    self.persistence
                        .ignored_users
                        .insert(target_user_id.clone());
                    ui.close_menu();
                }

                ui.menu_button("Triggers", |ui| {
                    #[derive(Clone, PartialEq)]
                    enum TriggerState {
                        Disabled,
                        Enabled,
                        EnabledOnce,
                    }

                    for (_, trigger) in self
                        .trigger_manager()
                        .triggers_iter_mut()
                        .filter(|(_, t)| t.requestable)
                    {
                        let mut current_state = match trigger.requested_users.get(target_user_id) {
                            Some(BehaviorOnTrigger::NoAction) => TriggerState::Enabled,
                            Some(BehaviorOnTrigger::Remove) => TriggerState::EnabledOnce,
                            None => TriggerState::Disabled,
                        };

                        ui.horizontal(|ui| {
                            ui.label(trigger.name.clone());
                            if ui
                                .selectable_value(&mut current_state, TriggerState::Disabled, "Off")
                                .clicked()
                            {
                                trigger.requested_users.remove(target_user_id);
                            }

                            if ui
                                .selectable_value(&mut current_state, TriggerState::Enabled, "On")
                                .clicked()
                            {
                                trigger
                                    .requested_users
                                    .insert(target_user_id.clone(), BehaviorOnTrigger::NoAction);
                            }

                            if ui
                                .selectable_value(
                                    &mut current_state,
                                    TriggerState::EnabledOnce,
                                    "On (once)",
                                )
                                .clicked()
                            {
                                trigger
                                    .requested_users
                                    .insert(target_user_id.clone(), BehaviorOnTrigger::Remove);
                            }
                        });
                    }
                });

                if ui.button("View Raw Data").clicked() {
                    self.received_data_viewer.show_data(
                        user_status.into(),
                        format!("Raw Data for {}", user_status.display_name()),
                        user_status.user_id.clone(),
                    );
                    ui.close_menu();
                }
            }
            None => {
                ui.label("[[No Options]]");
            }
        }
    }

    fn get_user_display_name(&self, user_id: &UniqueUserId) -> Option<String> {
        self.current_status.get(user_id).map(|s| s.display_name())
    }

    fn trigger_manager(&mut self) -> &mut TriggerManager {
        &mut self.persistence.trigger_manager
    }
}

fn load_and_migrate_persistence(
    cc: &eframe::CreationContext<'_>,
    log_file_location: &PathBuf,
) -> Persistence {
    if let Some(v) = cc.storage.and_then(|storage| {
        eframe::get_value::<VersionedPersistence>(storage, Persistence::STORAGE_KEY)
    }) {
        v.into()
    } else if let Some(p) = cc
        .storage
        .and_then(|storage| eframe::get_value::<PersistenceV1>(storage, Persistence::STORAGE_KEY))
    {
        let v1 = VersionedPersistence::V1(p);
        v1.into()
    } else {
        // This is here as insurance to ensure we don't overwrite a previously valid .ron with empty Persistence if Persistence wasn't migrated correctly
        OSNotificationDispatch.show_notification("Gwaihir init failed", "Gwaihir failed to initialize due to an error decoding its config. View the log for more details");
        open_log_file(log_file_location);
        panic!("Failed to deserialize app config");
    }
}

fn open_log_file(log_file_location: impl AsRef<OsStr>) {
    opener::open(log_file_location).log_expect("Failed to open file using default OS handler");
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
                self.transmission_spy
                    .set_data(TimestampedData::now(sensor_outputs.clone()));
                self.network.publish_update(sensor_outputs);
            }
        }

        self.network.try_reconnect_if_needed();
        while let Ok(update) = self.network.try_recv() {
            match update {
                RemoteUpdate::UserStatusUpdated(status) => {
                    if self.subscribed_to_user(&status.user_id) {
                        debug!("Got user update from DB: {:#?}", &status);
                        if let Some(current) = self.current_status.get(&status.user_id) {
                            let display_name = self
                                .get_user_display_name(&status.user_id)
                                .unwrap_or_else(|| "Unknown".to_string());
                            self.persistence.trigger_manager.execute_triggers(
                                &status.user_id,
                                display_name,
                                Update::new(&current.sensor_outputs, &status.sensor_outputs),
                                &OSNotificationDispatch,
                                &mut self.user_summaries,
                            );
                        }
                        self.current_status.insert(status.user_id.clone(), status);
                    }
                }
            };
        }

        if self.current_user_id.is_none() {
            self.current_user_id = self.network.get_current_user_id();
            self.transmission_spy
                .set_user_id(self.current_user_id.clone());
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

                    ui.menu_button("Open", |ui| {
                        if ui.button("Log").clicked() {
                            open_log_file(self.log_file_location.clone());
                            ui.close_menu();
                        }

                        if ui.button("Data Directory").clicked() {
                            opener::open(project_dirs().data_dir()).log_expect(
                                "Failed to open data directory using default OS handler",
                            );
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Manage", |ui| {
                        if ui.button("Network").clicked() {
                            self.network_window.set_shown(true);
                            ui.close_menu();
                        }

                        if ui.button("Triggers").clicked() {
                            self.triggers_window.set_shown(true);
                            ui.close_menu();
                        }
                    });

                    ui.menu_button("Users", |ui| {
                        if ui.button("View Sent Data").clicked() {
                            self.transmission_spy.set_shown(true);
                            ui.close_menu();
                        }

                        if ui.button("Clear ignored users").clicked() {
                            self.persistence.ignored_users.clear();
                            ui.close_menu();
                        }

                        if ui.button("Create Fake User").clicked() {
                            self.add_fake_user_window.set_shown(true);
                            ui.close_menu();
                        }
                    });

                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });

                if cfg!(debug_assertions) {
                    ui.separator();
                    ui.label(format!("Frame: {}", ctx.frame_nr()));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.network.is_offline() {
                ui.label(RichText::new("⚠⚠ OFFLINE ⚠⚠").heading().color(Color32::RED));
            }

            let user_status_list = self.get_filtered_sorted_user_statuses();
            ScrollArea::vertical().show(ui, |ui| {
                for (id, status) in user_status_list.iter() {
                    let summary = self.user_summaries.get(id);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        if summary.is_some() {
                            let online = status
                                .last_update
                                .signed_duration_since(Utc::now())
                                .num_minutes()
                                < 6;
                            OnlineStatus { online }
                                .show(ui, id)
                                .on_hover_text_at_pointer(last_updated_text(status));
                        } else {
                            status.sensor_outputs.show_first(
                                |o| matches!(o, SensorOutput::OnlineStatus(_)),
                                ui,
                                id,
                            );
                        }
                        ui.heading(status.display_name())
                            .context_menu(|ui| {
                                self.show_user_context_menu(id, ui, status);
                            })
                            .on_hover_text_at_pointer("Right click for options");
                        if summary.is_none() {
                            ui.label(RichText::new(format!(
                                " {} ",
                                HumanTime::from(status.last_update)
                            )))
                            .on_hover_text_at_pointer(last_updated_text(status));
                        }
                    });

                    if let Some(summary) = summary {
                        egui::CollapsingHeader::new(RichText::new(summary).size(15.0))
                            .id_source(format!("{}_details", id))
                            .show(ui, |ui| {
                                show_sensor_status(status, ui, id);
                            });
                    } else {
                        show_sensor_status(status, ui, id);
                    }
                }
            });

            egui::warn_if_debug_build(ui);
        });

        self.network_window
            .show(ctx, &mut self.network, &mut self.persistence, || {
                self.current_status.clear();
            });
        self.transmission_spy.show(ctx);
        self.received_data_viewer.show(ctx);
        self.add_fake_user_window.show(ctx, |user_status| {
            self.network
                .queue_fake_update(RemoteUpdate::UserStatusUpdated(user_status))
                .log_expect("Failed to queue fake user update");
        });
        self.triggers_window
            .show(ctx, &mut self.persistence.trigger_manager);
    }
}

fn last_updated_text(status: &UserStatus<SensorOutputs>) -> String {
    format!(
        "Last updated: {}",
        nicely_formatted_datetime(status.last_update.with_timezone(&Local),)
    )
}

fn show_sensor_status(status: &UserStatus<SensorOutputs>, ui: &mut egui::Ui, id: &UniqueUserId) {
    status
        .sensor_outputs
        .show_first(|o| matches!(o, SensorOutput::LockStatus(_)), ui, id);

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

    status
        .sensor_outputs
        .show_first(|o| matches!(o, SensorOutput::MicrophoneUsage(_)), ui, id);
}

impl GwaihirApp {
    fn subscribed_to_user(&self, user_id: &UniqueUserId) -> bool {
        !self.persistence.ignored_users.contains(user_id)
    }
}
