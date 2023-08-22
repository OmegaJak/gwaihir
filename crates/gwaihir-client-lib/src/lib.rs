use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use chrono;

pub enum RemoteUpdate {
    UserStatusUpdated(String, SensorData, DateTime<Utc>),
}

pub trait NetworkInterface {
    fn new() -> Self;
    fn publish_status_update(&self, status: SensorData);
    fn receive_updates(&self) -> Vec<RemoteUpdate>;
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MicrophoneUsage {
    pub app_name: String,
    pub last_used: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct SensorData {
    pub num_locks: u32,
    pub num_unlocks: u32,
    pub microphone_usage: Vec<MicrophoneUsage>,
}

impl Default for SensorData {
    fn default() -> Self {
        Self {
            num_locks: 0,
            num_unlocks: 0,
            microphone_usage: Vec::new(),
        }
    }
}
