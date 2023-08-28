use chrono::{DateTime, Utc};
use nutype::nutype;
use serde::{Deserialize, Serialize};

pub use chrono;

#[cfg(debug_assertions)]
pub const APP_ID: &str = "gwaihir-debug";

#[cfg(not(debug_assertions))]
pub const APP_ID: &str = "gwaihir";

#[nutype(derive(
    Deref,
    AsRef,
    Clone,
    Into,
    Hash,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
))]
pub struct UniqueUserId(String);

#[nutype(derive(AsRef, Clone, Into))]
pub struct Username(String);

pub enum RemoteUpdate<T> {
    UserStatusUpdated(UserStatus<T>),
}

#[derive(Clone)]
pub struct UserStatus<T> {
    pub user_id: UniqueUserId,
    pub username: Username,
    pub last_update: DateTime<Utc>,
    pub is_online: bool,
    pub sensor_outputs: T,
}

impl<T> UserStatus<T> {
    pub fn display_name(&self) -> String {
        if self.username.as_ref().is_empty() {
            self.user_id.clone().into()
        } else {
            self.username.clone().into()
        }
    }
}

pub trait AcceptsOnlineStatus {
    fn set_online_status(&mut self, online: bool);
}

pub trait NetworkInterface<T>
where
    T: Serialize + for<'a> Deserialize<'a>,
{
    fn new(update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static) -> Self;
    fn publish_update(&self, sensor_outputs: T);
    fn set_username(&self, name: String);
    fn get_current_user_id(&self) -> Option<UniqueUserId>;
}

// #[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
// pub struct SensorData {
//     pub num_locks: u32,
//     pub num_unlocks: u32,
//     pub microphone_usage: Vec<MicrophoneUsage>,
// }

// impl Default for SensorData {
//     fn default() -> Self {
//         Self {
//             num_locks: 0,
//             num_unlocks: 0,
//             microphone_usage: Vec::new(),
//         }
//     }
// }
