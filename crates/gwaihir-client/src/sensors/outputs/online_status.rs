use egui::{Color32, RichText};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct OnlineStatus {
    pub online: bool,
}

impl SensorWidget<egui::Response> for OnlineStatus {
    fn show(&self, ui: &mut egui::Ui, _id: &UniqueUserId) -> egui::Response {
        let mut online_color = Color32::RED;
        if self.online {
            online_color = Color32::GREEN;
        }
        ui.label(RichText::new("⏺ ").color(online_color).heading())
    }
}
