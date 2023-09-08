use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

use super::{
    lock_status::LockStatus, microphone_usage::MicrophoneUsage, online_status::OnlineStatus,
    summarized_window_activity::SummarizedWindowActivity,
};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SensorOutput {
    Empty,
    LockStatus(LockStatus),
    MicrophoneUsage(MicrophoneUsage),
    OnlineStatus(OnlineStatus),
    SummarizedWindowActivity(SummarizedWindowActivity),
}

pub trait SensorWidget {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId);
}

impl SensorWidget for SensorOutput {
    fn show(&self, ui: &mut egui::Ui, id: &UniqueUserId) {
        match self {
            SensorOutput::Empty => (),
            SensorOutput::LockStatus(status) => status.show(ui, id),
            SensorOutput::MicrophoneUsage(usage) => usage.show(ui, id),
            SensorOutput::OnlineStatus(online) => online.show(ui, id),
            SensorOutput::SummarizedWindowActivity(activity) => activity.show(ui, id),
        }
    }
}

impl SensorOutput {
    pub fn should_send_to_remote(&self) -> bool {
        match self {
            SensorOutput::Empty => false,
            SensorOutput::LockStatus(_) => false,
            SensorOutput::MicrophoneUsage(_) => true,
            SensorOutput::OnlineStatus(_) => true,
            SensorOutput::SummarizedWindowActivity(_) => true,
        }
    }
}
