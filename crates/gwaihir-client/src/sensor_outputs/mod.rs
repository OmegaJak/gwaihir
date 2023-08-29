use gwaihir_client_lib::{AcceptsOnlineStatus, UniqueUserId};
use serde::{Deserialize, Serialize};

use self::{
    lock_status::LockStatus, microphone_usage::MicrophoneUsage, online_status::OnlineStatus,
};

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

impl SensorOutputs {
    pub fn show_first(
        &self,
        mut predicate: impl FnMut(&SensorOutput) -> bool,
        ui: &mut egui::Ui,
        id: &UniqueUserId,
    ) {
        self.outputs
            .iter()
            .find(|v| predicate(v))
            .map(|o| o.show(ui, id));
    }

    fn has_online_status(&self) -> bool {
        self.outputs
            .iter()
            .any(|o| matches!(o, SensorOutput::OnlineStatus(_)))
    }
}

impl AcceptsOnlineStatus for SensorOutputs {
    fn set_online_status(&mut self, online: bool) {
        if self.has_online_status() {
            panic!("We attempted to set the online status when we already had one");
            //TODO: Should we be panicking here?
        }

        self.outputs
            .push(SensorOutput::OnlineStatus(OnlineStatus { online }))
    }
}
