use gwaihir_client_lib::{
    chrono::{DateTime, Utc},
    NetworkInterface, NetworkInterfaceCreator, UniqueUserId, UserStatus, Username,
};

pub struct OfflineNetworkInterface<T> {
    update_callback: Box<dyn Fn(gwaihir_client_lib::RemoteUpdate<T>) + Send>,
}

impl<T> NetworkInterfaceCreator<T, OfflineNetworkInterface<T>> for OfflineNetworkInterface<T> {
    fn new(
        update_callback: impl Fn(gwaihir_client_lib::RemoteUpdate<T>) + Send + Clone + 'static,
    ) -> Self {
        Self {
            update_callback: Box::new(update_callback),
        }
    }
}

impl<T> NetworkInterface<T> for OfflineNetworkInterface<T> {
    fn publish_update(&self, sensor_outputs: T) {
        (self.update_callback)(gwaihir_client_lib::RemoteUpdate::UserStatusUpdated(
            UserStatus {
                user_id: UniqueUserId::new("1234abcd"),
                username: Username::new("⚠⚠ OFFLINE ⚠⚠"),
                last_update: DateTime::<Utc>::MIN_UTC,
                sensor_outputs,
            },
        ));
    }

    fn set_username(&self, _name: String) {}

    fn get_current_user_id(&self) -> Option<gwaihir_client_lib::UniqueUserId> {
        None
    }
}
