use super::widgets::show_centered_window;
use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use egui::{ScrollArea, TextEdit};
use gwaihir_client_lib::{
    chrono::{DateTime, Local},
    UniqueUserId, UserStatus,
};

#[derive(Clone)]
pub struct TimestampedData<T> {
    time: DateTime<Local>,
    data: T,
}

impl<T> TimestampedData<T> {
    pub fn now(data: T) -> Self {
        Self {
            time: Local::now(),
            data,
        }
    }
}

impl From<&UserStatus<SensorOutputs>> for TimestampedData<SensorOutputs> {
    fn from(value: &UserStatus<SensorOutputs>) -> Self {
        Self {
            time: value.last_update.into(),
            data: value.sensor_outputs.clone(),
        }
    }
}

pub struct RawDataWindow {
    shown: bool,
    user_id: Option<UniqueUserId>,
    data: Option<TimestampedData<SensorOutputs>>,
    window_title: String,
}

impl RawDataWindow {
    pub fn new(title: String) -> Self {
        Self {
            shown: false,
            user_id: None,
            data: None,
            window_title: title,
        }
    }

    pub fn show_data(
        &mut self,
        data: TimestampedData<SensorOutputs>,
        window_title: String,
        user_id: UniqueUserId,
    ) {
        self.set_shown(true);
        self.set_data(data);
        self.set_user_id(Some(user_id));
        self.window_title = window_title;
    }

    pub fn set_data(&mut self, data: TimestampedData<SensorOutputs>) {
        self.data = Some(data);
    }

    pub fn set_user_id(&mut self, user_id: Option<UniqueUserId>) {
        self.user_id = user_id;
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.shown = show_centered_window(self.shown, self.window_title.clone(), ctx, |ui| {
            if let Some(TimestampedData { time, data }) = self.data.as_ref() {
                ui.label(format!("Data from {}", time));
                if let Some(user_id) = &self.user_id {
                    ui.horizontal(|ui| {
                        ui.label("User ID: ");
                        let mut user_id = user_id.to_string();
                        ui.text_edit_singleline(&mut user_id);
                    });
                }
                if let Ok(mut text) = serde_json::to_string_pretty(data) {
                    ScrollArea::vertical().show(ui, |ui| {
                        TextEdit::multiline(&mut text).show(ui);
                    });
                } else {
                    ui.label("Failed to serialize...");
                }
            } else {
                ui.label("No data yet");
            }
        });
    }
}
