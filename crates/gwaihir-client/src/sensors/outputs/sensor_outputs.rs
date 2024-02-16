use gwaihir_client_lib::{
    chrono::{Duration, Utc},
    AcceptsOnlineStatus,
};
use log::error;
use serde::{Deserialize, Serialize};

use super::{
    keyboard_mouse_activity::KeyboardMouseActivity, microphone_usage::MicrophoneUsage,
    online_status::OnlineStatus, sensor_output::SensorOutput,
    summarized_window_activity::SummarizedWindowActivity,
};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SensorOutputs {
    pub outputs: Vec<SensorOutput>,
}

macro_rules! match_variant {
    ($value:expr, $variant:path) => {
        match $value {
            $variant(x) => Some(x),
            _ => None,
        }
    };
}

impl SensorOutputs {
    fn has_online_status(&self) -> bool {
        self.outputs
            .iter()
            .any(|o| matches!(o, SensorOutput::OnlineStatus(_)))
    }

    pub fn is_locked(&self) -> Option<bool> {
        self.find_summarized_window_activity()
            .map(|a| a.is_locked())
    }

    pub fn get_total_keyboard_mouse_usage(&self) -> Option<f64> {
        self.find_keyboard_mouse_activity()
            .and_then(|a| a.is_full().then_some(a.get_total_usage()))
    }

    pub fn get_num_apps_using_microphone(&self) -> Option<usize> {
        self.find_microphone_usage().map(|u| u.usage.len())
    }

    pub fn active_window_duration(&self) -> Option<Duration> {
        self.find_summarized_window_activity()
            .map(|a| Utc::now().signed_duration_since(a.current_window.started_using))
    }

    pub fn find_online_status(&self) -> Option<&OnlineStatus> {
        self.find_sensor_output(|o| match_variant!(o, SensorOutput::OnlineStatus))
    }

    pub fn find_summarized_window_activity(&self) -> Option<&SummarizedWindowActivity> {
        self.find_sensor_output(|o| match_variant!(o, SensorOutput::SummarizedWindowActivity))
    }

    pub fn find_keyboard_mouse_activity(&self) -> Option<&KeyboardMouseActivity> {
        self.find_sensor_output(|o| match_variant!(o, SensorOutput::KeyboardMouseActivity))
    }

    pub fn find_microphone_usage(&self) -> Option<&MicrophoneUsage> {
        self.find_sensor_output(|o| match_variant!(o, SensorOutput::MicrophoneUsage))
    }

    pub fn find_sensor_output<R>(&self, f: impl FnMut(&SensorOutput) -> Option<&R>) -> Option<&R> {
        self.outputs.iter().find_map(f)
    }
}

impl AcceptsOnlineStatus for SensorOutputs {
    fn set_online_status(&mut self, online: bool) {
        if self.has_online_status() {
            error!("We attempted to set the online status when we already had one");
        }

        self.outputs
            .push(SensorOutput::OnlineStatus(OnlineStatus { online }))
    }
}
