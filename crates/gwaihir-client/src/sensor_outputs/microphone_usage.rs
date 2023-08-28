use egui::CollapsingHeader;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct MicrophoneUsage {
    pub usage: Vec<AppMicrophoneUsage>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct AppMicrophoneUsage {
    pub app_name: String,
    pub last_used: u64,
}

impl Default for MicrophoneUsage {
    fn default() -> Self {
        Self { usage: Vec::new() }
    }
}

impl SensorWidget for MicrophoneUsage {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        CollapsingHeader::new("Microphone Usage")
            .default_open(true)
            .id_source(format!("{}_mic", id.as_ref()))
            .show(ui, |ui| {
                ui.label(format!(
                    "{} app(s) currently listening to the microphone",
                    self.usage.len()
                ));
                for usage in self.usage.iter() {
                    let pretty_name = usage.app_name.replace("#", "\\");
                    ui.label(pretty_name);
                }
            });
    }
}
