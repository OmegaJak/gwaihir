use gwaihir_client_lib::{AcceptsOnlineStatus, UniqueUserId};
use serde::{Deserialize, Serialize};

use super::{
    online_status::OnlineStatus,
    sensor_output::{SensorOutput, SensorWidget},
};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SensorOutputs {
    pub outputs: Vec<SensorOutput>,
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
            eprintln!("We attempted to set the online status when we already had one");
        }

        self.outputs
            .push(SensorOutput::OnlineStatus(OnlineStatus { online }))
    }
}
