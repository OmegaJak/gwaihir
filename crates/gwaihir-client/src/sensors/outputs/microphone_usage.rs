use egui::CollapsingHeader;
use gwaihir_client_lib::UniqueUserId;
use nutype::nutype;
use serde::{Deserialize, Serialize};

use crate::sensors::outputs::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MicrophoneUsage {
    pub usage: Vec<AppName>,
}

#[nutype(derive(Serialize, Deserialize, PartialEq, AsRef, Clone, Into, Debug, From))]
pub struct AppName(String);

impl SensorWidget<()> for MicrophoneUsage {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        CollapsingHeader::new("Microphone Usage")
            .default_open(true)
            .id_source(format!("{}_mic", id.as_ref()))
            .show(ui, |ui| {
                ui.label(format!(
                    "{} app(s) currently listening to the microphone:",
                    self.usage.len()
                ));
                for app in self.usage.iter() {
                    let pretty_name = app.as_ref().replace('#', "\\");
                    ui.label(pretty_name);
                }
            });
    }
}
