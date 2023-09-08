use chrono::{DateTime, Utc};
use nutype::nutype;

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
    Debug
))]
pub struct UniqueUserId(String);

#[nutype(derive(AsRef, Clone, Into, Debug))]
pub struct Username(String);

pub enum RemoteUpdate<T> {
    UserStatusUpdated(UserStatus<T>),
    Disconnected,
}

#[derive(Clone, Debug)]
pub struct UserStatus<T> {
    pub user_id: UniqueUserId,
    pub username: Username,
    pub last_update: DateTime<Utc>,
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

pub trait NetworkInterfaceCreator<T, NI>
where
    NI: NetworkInterface<T>,
{
    fn new(
        update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static,
        on_disconnect_callback: impl FnOnce() + Send + 'static,
    ) -> NI;
}

pub trait NetworkInterface<T> {
    fn publish_update(&self, sensor_outputs: T);
    fn set_username(&self, name: String);
    fn get_current_user_id(&self) -> Option<UniqueUserId>;
}
