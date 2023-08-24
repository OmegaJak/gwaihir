use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

pub use chrono;

#[nutype]
#[derive(AsRef, Clone, Into, Hash, Eq, PartialEq)]
pub struct UniqueUserId(String);

#[nutype]
#[derive(AsRef, Clone, Into)]
pub struct Username(String);

pub enum RemoteUpdate {
    UserStatusUpdated(UniqueUserId, Username, bool, SensorData, DateTime<Utc>),
}

pub trait NetworkInterface {
    fn new(update_callback: impl Fn(RemoteUpdate) + Send + Clone + 'static) -> Self;
    fn publish_status_update(&self, status: SensorData);
    fn set_username(&self, name: String);
    fn get_current_user_id(&self) -> Option<UniqueUserId>;
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
