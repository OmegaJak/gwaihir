use egui::{CollapsingHeader, Color32, RichText};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use crate::{
    sensors::outputs::sensor_output::SensorWidget, ui_extension_methods::UIExtensionMethods,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct LockStatus {
    pub num_locks: u32,
    pub num_unlocks: u32,
}

impl Default for LockStatus {
    fn default() -> Self {
        Self {
            num_locks: 0,
            num_unlocks: 0,
        }
    }
}

impl SensorWidget for LockStatus {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        self.show_details(id, ui);
    }
}

impl LockStatus {
    fn show_details(&self, id: &UniqueUserId, ui: &mut egui::Ui) {
        let mut header_text = RichText::new("Currently Unlocked").color(Color32::DARK_GREEN);
        if self.num_locks > self.num_unlocks {
            header_text = RichText::new("Currently Locked").color(Color32::RED);
        }

        let layout_job = ui.create_default_layout_job(vec![header_text]);

        CollapsingHeader::new(layout_job)
            .id_source(format!("{}_locks", id.as_ref()))
            .show(ui, |ui| {
                ui.label(format!("Times Locked: {}", self.num_locks));
                ui.label(format!("Times Unlocked: {}", self.num_unlocks));
            });
    }
}
