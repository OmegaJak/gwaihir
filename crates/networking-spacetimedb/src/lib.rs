mod module_bindings;

use gwaihir_client_lib::{
    chrono::{DateTime, NaiveDateTime, Utc},
    AcceptsOnlineStatus, NetworkInterface, NetworkInterfaceCreator, RemoteUpdate, UniqueUserId,
    UserStatus, Username, APP_ID,
};
use log::error;
use module_bindings::*;
use serde::{Deserialize, Serialize};
use spacetimedb_sdk::{
    identity::{
        identity, load_credentials, once_on_connect, save_credentials, Credentials, Identity,
    },
    subscribe,
    table::{TableType, TableWithPrimaryKey},
};

/// The URL of the SpacetimeDB instance hosting our chat module.
const SPACETIMEDB_URI: &str = "https://testnet.spacetimedb.com";

/// The module name we chose when we published our module.
const DB_NAME: &str = "gwaihir-test2";

pub struct SpacetimeDBInterface {}

impl<T> NetworkInterfaceCreator<T, SpacetimeDBInterface> for SpacetimeDBInterface
where
    T: Serialize + for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    fn new(update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static) -> Self {
        register_callbacks(update_callback);
        connect_to_db();
        subscribe_to_tables();

        Self {}
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
}

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks<T>(update_callback: impl Fn(RemoteUpdate<T>) + Send + Clone + 'static)
where
    T: for<'a> Deserialize<'a> + AcceptsOnlineStatus,
{
    // // When we receive our `Credentials`, save them to a file.
    once_on_connect(on_connected);

    let callback_clone = update_callback.clone();
    User::on_insert(move |a, _| {
        if let Some(update) = convert_to_remote_update(a) {
            callback_clone(update);
        }
    });

    // When a user's status changes, print a notification.
    User::on_update(move |a, b, c| {
        if let Some(update) = on_user_updated(a, b, c) {
            update_callback(update);
        }
    });

    // // When we receive the message backlog, print it in timestamp order.
    // on_subscription_applied(on_sub_applied);

    // // When we fail to set our name, print a warning.
    // on_set_name(on_name_set);
}

/// Load credentials from a file and connect to the database.
fn connect_to_db() {
    connect(
        SPACETIMEDB_URI,
        DB_NAME,
        load_credentials(&creds_dir()).expect("Error reading stored credentials"),
    )
    .expect("Failed to connect");
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
                let last_update = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_micros(
                        new.last_status_update.unwrap().try_into().unwrap(),
                    )
                    .unwrap(),
                    Utc,
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
