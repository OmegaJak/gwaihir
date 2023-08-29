use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::{
    lock_status::LockStatus, microphone_usage::MicrophoneUsage, online_status::OnlineStatus,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SensorOutput {
    LockStatus(LockStatus),
    MicrophoneUsage(MicrophoneUsage),
    OnlineStatus(OnlineStatus),
}

pub trait SensorWidget {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId);
}

impl SensorWidget for SensorOutput {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        match self {
            SensorOutput::LockStatus(status) => status.show(ui, id),
            SensorOutput::MicrophoneUsage(usage) => usage.show(ui, id),
            SensorOutput::OnlineStatus(online) => online.show(ui, id),
        }
    }
}
