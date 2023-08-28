use egui::{Color32, RichText};

use super::SensorWidget;

pub struct OnlineStatus {
    online: bool,
}

impl SensorWidget for OnlineStatus {
    fn show(&self, ui: &mut egui::Ui, id: gwaihir_client_lib::UniqueUserId) {
        let mut online_color = Color32::RED;
        if self.online {
            online_color = Color32::GREEN;
        }
        ui.label(RichText::new("⏺ ").color(online_color).heading());
    }
}
