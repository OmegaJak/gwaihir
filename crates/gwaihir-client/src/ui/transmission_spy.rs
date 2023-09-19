use egui::{Align2, ScrollArea, TextEdit};
use gwaihir_client_lib::chrono::{DateTime, Local};

use crate::sensors::outputs::sensor_outputs::SensorOutputs;

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

    pub fn show(&self, ctx: &egui::Context) {
        let mut shown = self.shown;
        egui::Window::new("Last Sent Update")
            .pivot(Align2::CENTER_CENTER)
            .default_pos(ctx.screen_rect().center())
            .open(&mut shown)
            .show(ctx, |ui| {
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
