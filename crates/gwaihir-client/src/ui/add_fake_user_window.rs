use egui::{Color32, RichText, ScrollArea};
use gwaihir_client_lib::{
    chrono::{DateTime, FixedOffset, Utc},
    UniqueUserId, UserStatus, Username,
};

use crate::sensors::outputs::sensor_outputs::SensorOutputs;

use super::widgets::show_centered_window;

pub struct AddFakeUserWindow {
    shown: bool,
    id_input: String,
    username_input: String,
    use_custom_update_time: bool,
    custom_update_time: String,
    json_input: String,
    error_msg: Option<String>,
}

impl AddFakeUserWindow {
    pub fn new() -> Self {
        Self {
            shown: false,
            id_input: String::new(),
            username_input: String::new(),
            use_custom_update_time: false,
            custom_update_time: String::new(),
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

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.use_custom_update_time, "Custom Update Time");
                if self.use_custom_update_time {
                    ui.text_edit_singleline(&mut self.custom_update_time);
                }
            });

            if ui.button("Add Fake User").clicked() {
                let sensor_outputs = serde_json::from_str(&self.json_input);
                match sensor_outputs {
                    Ok(sensor_outputs) => {
                        let id = UniqueUserId::new(self.id_input.clone());
                        let username = Username::new(self.username_input.clone());
                        let last_update: DateTime<Utc> = if self.use_custom_update_time {
                            DateTime::<FixedOffset>::parse_from_rfc3339(&self.custom_update_time)
                                .unwrap()
                                .into()
                        } else {
                            Utc::now()
                        };
                        add_fake_user(UserStatus {
                            user_id: id,
                            username,
                            last_update,
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
