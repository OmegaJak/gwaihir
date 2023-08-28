use gwaihir_client_lib::UniqueUserId;

use self::{lock_status::LockStatus, microphone_usage::MicrophoneUsage};

pub mod lock_status;
pub mod microphone_usage;
pub mod online_status;

pub struct SensorOutputs {
    user_id: UniqueUserId,
    outputs: Vec<SensorOutput>,
}

pub enum SensorOutput {
    LockStatus(LockStatus),
    MicrophoneUsage(MicrophoneUsage),
}

pub trait SensorWidget {
    fn show(&self, ui: &mut egui::Ui, id: UniqueUserId);
}
