use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use self::{lock_status::LockStatus, microphone_usage::MicrophoneUsage};

pub mod lock_status;
pub mod microphone_usage;
pub mod online_status;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct SensorOutputs {
    pub outputs: Vec<SensorOutput>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum SensorOutput {
    LockStatus(LockStatus),
    MicrophoneUsage(MicrophoneUsage),
}

pub trait SensorWidget {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId);
}

impl SensorWidget for SensorOutput {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        match self {
            SensorOutput::LockStatus(status) => status.show(ui, id),
            SensorOutput::MicrophoneUsage(usage) => usage.show(ui, id),
        }
    }
}
