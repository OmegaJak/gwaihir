use egui::{CollapsingHeader, Color32, RichText};
use gwaihir_client_lib::UniqueUserId;

use super::SensorWidget;

pub struct LockStatus {
    num_locks: u32,
    num_unlocks: u32,
}

impl SensorWidget for LockStatus {
    fn show(&self, ui: &mut egui::Ui, id: UniqueUserId) {
        self.show_overall(ui);
        self.show_details(id, ui);
    }
}

impl LockStatus {
    fn show_overall(&self, ui: &mut egui::Ui) {
        if self.num_locks > self.num_unlocks {
            ui.label(RichText::new("Currently Locked").color(Color32::RED));
        } else {
            ui.label(RichText::new("Currently Unlocked").color(Color32::DARK_GREEN));
        }
    }

    fn show_details(&self, id: UniqueUserId, ui: &mut egui::Ui) {
        CollapsingHeader::new("Locks/Unlocks")
            .id_source(format!("{}_locks", id.as_ref()))
            .show(ui, |ui| {
                ui.label(format!("Times Locked: {}", self.num_locks));
                ui.label(format!("Times Unlocked: {}", self.num_unlocks));
            });
    }
}
