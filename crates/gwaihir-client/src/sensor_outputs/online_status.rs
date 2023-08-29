use egui::{Color32, RichText};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct OnlineStatus {
    pub online: bool,
}

impl SensorWidget for OnlineStatus {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        let mut online_color = Color32::RED;
        if self.online {
            online_color = Color32::GREEN;
        }
        ui.label(RichText::new("⏺ ").color(online_color).heading());
    }
}
