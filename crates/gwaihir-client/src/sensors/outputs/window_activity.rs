use gwaihir_client_lib::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::sensor_output::SensorWidget;

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WindowActivity {
    pub current_window: ActiveWindow,
    pub previously_active_windows: Vec<PreviouslyActiveWindow>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ActiveWindow {
    pub app_name: String,
    pub started_using: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PreviouslyActiveWindow {
    pub app_name: String,
    pub started_using: DateTime<Utc>,
    pub stopped_using: DateTime<Utc>,
}

pub trait RepresentsWindow {
    fn app_name(&self) -> &str;
}

pub trait SameWindow {
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool;
}

impl SensorWidget for WindowActivity {
    fn show(&self, ui: &mut egui::Ui, _id: &gwaihir_client_lib::UniqueUserId) {
        ui.label("HELLOO??");
        ui.label(format!("{:#?}", self));
    }
}

impl From<active_win_pos_rs::ActiveWindow> for ActiveWindow {
    fn from(window: active_win_pos_rs::ActiveWindow) -> Self {
        Self {
            app_name: window.app_name,
            started_using: Utc::now(),
        }
    }
}

impl RepresentsWindow for ActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl ActiveWindow {
    pub fn to_no_longer_active(self) -> PreviouslyActiveWindow {
        PreviouslyActiveWindow {
            app_name: self.app_name,
            started_using: self.started_using,
            stopped_using: Utc::now(),
        }
    }
}

impl RepresentsWindow for PreviouslyActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl RepresentsWindow for active_win_pos_rs::ActiveWindow {
    fn app_name(&self) -> &str {
        &self.app_name
    }
}

impl<T: RepresentsWindow> SameWindow for T {
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool {
        self.app_name() == other.app_name()
    }
}
