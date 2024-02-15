use egui::{CollapsingHeader, Color32, RichText};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use crate::sensors::outputs::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct LockStatus {
    pub num_locks: u32,
    pub num_unlocks: u32,
}

impl SensorWidget<()> for LockStatus {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        self.show_details(id, ui);
    }
}

impl LockStatus {
    pub fn is_locked(&self) -> bool {
        self.num_locks > self.num_unlocks
    }

    fn show_details(&self, id: &UniqueUserId, ui: &mut egui::Ui) {
        let header_text = if self.is_locked() {
            RichText::new("Currently Locked").color(Color32::RED)
        } else {
            RichText::new("Currently Unlocked").color(Color32::DARK_GREEN)
        };

        CollapsingHeader::new(header_text)
            .id_source(format!("{}_locks", id.as_ref()))
            .show(ui, |ui| {
                ui.label(format!("Times Locked: {}", self.num_locks));
                ui.label(format!("Times Unlocked: {}", self.num_unlocks));
            });
    }
}
