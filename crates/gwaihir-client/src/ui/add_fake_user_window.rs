use egui::{Color32, RichText, ScrollArea};
use gwaihir_client_lib::{chrono::Utc, UniqueUserId, UserStatus, Username};

use crate::sensors::outputs::sensor_outputs::SensorOutputs;

use super::widgets::show_centered_window;

pub struct AddFakeUserWindow {
    shown: bool,
    id_input: String,
    username_input: String,
    json_input: String,
    error_msg: Option<String>,
}

impl AddFakeUserWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            id_input: String::new(),
            username_input: String::new(),
            json_input: String::new(),
            error_msg: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        add_fake_user: impl FnOnce(UserStatus<SensorOutputs>),
    ) {
        self.shown = show_centered_window(self.shown, "Add Fake User", ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Id: ");
                ui.text_edit_singleline(&mut self.id_input);
            });

            ui.horizontal(|ui| {
                ui.label("Username: ");
                ui.text_edit_singleline(&mut self.username_input);
            });

            ui.horizontal(|ui| {
                ui.label("JSON: ");
                ScrollArea::vertical().show(ui, |ui| {
                    ui.text_edit_multiline(&mut self.json_input);
                });
            });

            if ui.button("Add Fake User").clicked() {
                let sensor_outputs = serde_json::from_str(&self.json_input);
                match sensor_outputs {
                    Ok(sensor_outputs) => {
                        let id = UniqueUserId::new(self.id_input.clone());
                        let username = Username::new(self.username_input.clone());
                        add_fake_user(UserStatus {
                            user_id: id,
                            username,
                            last_update: Utc::now(),
                            sensor_outputs,
                        });
                        self.error_msg = None;
                    }
                    Err(e) => {
                        self.error_msg = Some(e.to_string());
                    }
                }
            }

            if let Some(error_msg) = self.error_msg.as_ref() {
                ScrollArea::both().show(ui, |ui| {
                    ui.label(RichText::new(error_msg).color(Color32::RED));
                });
            }
        });
    }
}
