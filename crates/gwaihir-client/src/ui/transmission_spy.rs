use super::widgets::show_centered_window;
use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use egui::{ScrollArea, TextEdit};
use gwaihir_client_lib::chrono::{DateTime, Local};

pub struct TransmissionSpy {
    shown: bool,
    latest_update: Option<(DateTime<Local>, SensorOutputs)>,
}

impl TransmissionSpy {
    pub fn new() -> Self {
        Self {
            shown: false,
            latest_update: None,
        }
    }

    pub fn set_shown(&mut self, shown: bool) {
        self.shown = shown;
    }

    pub fn record_update(&mut self, update: SensorOutputs) {
        self.latest_update = Some((Local::now(), update));
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.shown = show_centered_window(self.shown, "Last Sent Update", ctx, |ui| {
            if let Some((update_time, data)) = self.latest_update.as_ref() {
                ui.label(format!("Sent at {}", update_time));
                if let Ok(mut text) = serde_json::to_string_pretty(data) {
                    ScrollArea::vertical().show(ui, |ui| {
                        TextEdit::multiline(&mut text).show(ui);
                    });
                } else {
                    ui.label("Failed to serialize...");
                }
            } else {
                ui.label("No updates sent yet");
            }
        });
    }
}
