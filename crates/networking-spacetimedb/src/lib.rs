mod module_bindings;

use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use gwaihir_client_lib::{
    chrono::{NaiveDateTime, TimeZone, Utc},
    AcceptsOnlineStatus, NetworkInterface, NetworkInterfaceCreator, RemoteUpdate, UniqueUserId,
    UserStatus, Username, APP_ID,
};
use log::{error, info, warn};
use module_bindings::*;
use serde::{Deserialize, Serialize};
use spacetimedb_sdk::{
    disconnect,
    identity::{
        identity, load_credentials, once_on_connect, save_credentials, Credentials, Identity,
    },
    on_disconnect, subscribe,
    table::{TableType, TableWithPrimaryKey},
};

/// The URL of the SpacetimeDB instance hosting our chat module.
const SPACETIMEDB_URI: &str = "https://testnet.spacetimedb.com";

pub struct SpacetimeDBInterface {
    is_connected: Arc<AtomicBool>,
    creation_parameters: SpacetimeDBCreationParameters,
}

pub struct SpacetimeDBCreationParameters {
    pub db_name: String,
}

impl<T> NetworkInterfaceCreator<T, SpacetimeDBInterface, SpacetimeDBCreationParameters>
    for SpacetimeDBInterface
where
    T: Serialize + for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    fn new(
        update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static,
        mut on_disconnect_callback: impl FnMut() + Send + 'static,
        creation_params: SpacetimeDBCreationParameters,
    ) -> Self {
        let mut interface = Self {
            is_connected: Arc::new(AtomicBool::new(false)),
            creation_parameters: creation_params,
        };
        let is_connected_clone = interface.is_connected.clone();
        register_callbacks(update_callback, move || {
            info!("Disconnected from SpacetimeDB!");
            is_connected_clone.store(false, atomic::Ordering::Relaxed);
            on_disconnect_callback();
        });
        interface.connect_to_db();
        subscribe_to_tables();

        interface
    }
}

impl<T> NetworkInterface<T> for SpacetimeDBInterface
where
    T: Serialize + for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    fn publish_update(&self, sensor_outputs: T) {
        let json = serde_json::to_string(&sensor_outputs).unwrap();
        set_status(json);
    }

    fn get_current_user_id(&self) -> Option<UniqueUserId> {
        Some(UniqueUserId::new(identity_leading_hex(&identity().ok()?)))
    }

    fn set_username(&self, name: String) {
        set_name(name)
    }

    fn get_network_type(&self) -> gwaihir_client_lib::NetworkType {
        gwaihir_client_lib::NetworkType::SpacetimeDB
    }

    fn is_connected(&self) -> bool {
        self.is_connected.load(atomic::Ordering::Relaxed)
    }

    fn try_reconnect(&mut self) -> bool {
        let connected = self.connect_to_db();
        if connected {
            subscribe_to_tables();
        }
        connected
    }
}

impl SpacetimeDBInterface {
    fn connect_to_db(&mut self) -> bool {
        match connect(
            SPACETIMEDB_URI,
            &self.creation_parameters.db_name,
            load_credentials(&creds_dir()).expect("Error reading stored credentials"),
        ) {
            Ok(_) => {
                self.is_connected.store(true, atomic::Ordering::Relaxed);
                true
            }
            Err(err) => {
                warn!("Failed to connect to SpacetimeDB: {}", err);
                false
            }
        }
    }
}

impl Drop for SpacetimeDBInterface {
    fn drop(&mut self) {
        log::debug!("Disconnecting from SpacetimeDB because the interface was dropped");
        disconnect()
    }
}

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks<T>(
    update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static,
    on_disconnect_callback: impl FnMut() + Send + 'static,
) where
    T: for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    // // When we receive our `Credentials`, save them to a file.
    once_on_connect(on_connected);
    on_disconnect(on_disconnect_callback);

    let callback_clone = update_callback.clone();
    User::on_insert(move |a, _| {
        if let Some(update) = convert_to_remote_update(a) {
            callback_clone(update);
        }
    });

    User::on_update(move |a, b, c| {
        if let Some(update) = on_user_updated(a, b, c) {
            update_callback(update);
        }
    });
}

/// Register subscriptions for all rows of both tables.
fn subscribe_to_tables() {
    subscribe(&["SELECT * FROM User;"]).unwrap();
}

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(creds: &Credentials) {
    if let Err(e) = save_credentials(&creds_dir(), creds) {
        error!("Failed to save credentials: {:?}", e);
    }
}

fn identity_leading_hex(id: &Identity) -> String {
    hex::encode(&id.bytes()[0..8])
}

/// Our `User::on_update` callback:
/// print a notification about name and status changes.
fn on_user_updated<T>(old: &User, new: &User, _: Option<&ReducerEvent>) -> Option<RemoteUpdate<T>>
where
    T: for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    if new.last_status_update != old.last_status_update
        || old.name != new.name
        || old.online != new.online
    {
        return convert_to_remote_update(new);
    }

    None
}

fn convert_to_remote_update<T>(new: &User) -> Option<RemoteUpdate<T>>
where
    T: for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    if let Some(status) = new.status.clone() {
        match serde_json::from_str::<T>(&status) {
            Ok(mut sensor_data) => {
                let last_update = Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_micros(
                        new.last_status_update.unwrap().try_into().unwrap(),
                    )
                    .unwrap(),
                );
                sensor_data.set_online_status(new.online);
                return Some(RemoteUpdate::UserStatusUpdated(UserStatus {
                    user_id: UniqueUserId::new(identity_leading_hex(&new.identity)),
                    username: Username::new(new.name.clone().unwrap_or_default()),
                    sensor_outputs: sensor_data,
                    last_update,
                }));
            }
            Err(e) => {
                error!(
                    "Failed to deserialize sensor data for user ({:?}, {}): {}",
                    new.name,
                    identity_leading_hex(&new.identity),
                    e
                );
            }
        }
    }

    None
}

fn creds_dir() -> String {
    format!(".{}", APP_ID)
}
