use serde::{Deserialize, Serialize};

use super::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyboardMouseActivity {
    pub keyboard_usage: f64,
    pub mouse_movement: f64,
    pub mouse_button_usage: f64,
}

impl SensorWidget for KeyboardMouseActivity {
    fn show(&self, ui: &mut egui::Ui, id: &gwaihir_client_lib::UniqueUserId) {
        ui.label(format!(
            "Keyboard: {}, Mouse Move: {}, Mouse Button: {}",
            self.keyboard_usage, self.mouse_movement, self.mouse_button_usage
        ));
    }
}
