use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::{
    keyboard_mouse_activity::KeyboardMouseActivity, lock_status::LockStatus,
    microphone_usage::MicrophoneUsage, online_status::OnlineStatus,
    summarized_window_activity::SummarizedWindowActivity,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SensorOutput {
    Empty,
    LockStatus(LockStatus),
    MicrophoneUsage(MicrophoneUsage),
    OnlineStatus(OnlineStatus),
    SummarizedWindowActivity(SummarizedWindowActivity),
    KeyboardMouseActivity(KeyboardMouseActivity),
}

pub trait SensorWidget<R> {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) -> R;
}

impl SensorOutput {
    pub fn should_send_to_remote(&self) -> bool {
        match self {
            SensorOutput::Empty => false,
            SensorOutput::LockStatus(_) => false,
            SensorOutput::MicrophoneUsage(_) => true,
            SensorOutput::OnlineStatus(_) => true,
            SensorOutput::SummarizedWindowActivity(_) => true,
            SensorOutput::KeyboardMouseActivity(_) => true,
        }
    }
}
