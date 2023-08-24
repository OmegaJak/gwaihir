mod module_bindings;

use gwaihir_client_lib::{
    chrono::{DateTime, NaiveDateTime, Utc},
    NetworkInterface, RemoteUpdate, SensorData, UniqueUserId, Username,
};
use module_bindings::*;
use spacetimedb_sdk::{
    identity::{load_credentials, once_on_connect, save_credentials, Credentials, Identity},
    reducer::Status,
    subscribe,
    table::TableWithPrimaryKey,
};

const CREDS_DIR: &str = ".gwaihir";

/// The URL of the SpacetimeDB instance hosting our chat module.
const SPACETIMEDB_URI: &str = "https://testnet.spacetimedb.com";

/// The module name we chose when we published our module.
const DB_NAME: &str = "gwaihir-test2";

pub struct SpacetimeDBInterface {}

impl NetworkInterface for SpacetimeDBInterface {
    fn new(update_callback: impl Fn(RemoteUpdate) + Send + 'static) -> Self {
        register_callbacks(update_callback);
        connect_to_db();
        subscribe_to_tables();

        Self {}
    }

    fn publish_status_update(&self, status: SensorData) {
        let json = serde_json::to_string(&status).unwrap();
        set_status(json);
    }
}

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks(update_callback: impl Fn(RemoteUpdate) + Send + 'static) {
    // // When we receive our `Credentials`, save them to a file.
    once_on_connect(on_connected);

    // // When a new user joins, print a notification.
    // User::on_insert(on_user_inserted);

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
        load_credentials(CREDS_DIR).expect("Error reading stored credentials"),
    )
    .expect("Failed to connect");
}

/// Register subscriptions for all rows of both tables.
fn subscribe_to_tables() {
    subscribe(&["SELECT * FROM User;"]).unwrap();
}

/// Our `on_connect` callback: save our credentials to a file.
fn on_connected(creds: &Credentials) {
    if let Err(e) = save_credentials(CREDS_DIR, creds) {
        eprintln!("Failed to save credentials: {:?}", e);
    }
}

/// Our `User::on_insert` callback:
/// if the user is online, print a notification.
fn on_user_inserted(user: &User, _: Option<&ReducerEvent>) {
    if user.online {
        println!("User {} connected.", user_name_or_identity(user));
    }
}

fn user_name_or_identity(user: &User) -> String {
    user.name
        .clone()
        .unwrap_or_else(|| identity_leading_hex(&user.identity))
}

fn identity_leading_hex(id: &Identity) -> String {
    hex::encode(&id.bytes()[0..8])
}

/// Our `User::on_update` callback:
/// print a notification about name and status changes.
fn on_user_updated(old: &User, new: &User, _: Option<&ReducerEvent>) -> Option<RemoteUpdate> {
    // if old.name != new.name {
    //     println!(
    //         "User {} renamed to {}.",
    //         user_name_or_identity(old),
    //         user_name_or_identity(new)
    //     );
    // }
    // if old.online && !new.online {
    //     println!("User {} disconnected.", user_name_or_identity(new));
    // }
    // if !old.online && new.online {
    //     println!("User {} connected.", user_name_or_identity(new));
    // }

    if new.last_status_update != old.last_status_update {
        if let Some(status) = new.status.clone() {
            let sensor_data = serde_json::from_str(&status).unwrap();
            let time = DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp_micros(
                    new.last_status_update.unwrap().try_into().unwrap(),
                )
                .unwrap(),
                Utc,
            );
            return Some(RemoteUpdate::UserStatusUpdated(
                UniqueUserId::new(identity_leading_hex(&new.identity)),
                Username::new(new.name.clone().unwrap_or_default()),
                sensor_data,
                time,
            ));
        }
    }

    None
}

/// Our `on_subscription_applied` callback:
/// sort all past messages and print them in timestamp order.
fn on_sub_applied() {
    // let mut messages = Message::iter().collect::<Vec<_>>();
    // messages.sort_by_key(|m| m.sent);
    // for message in messages {
    //     print_message(&message);
    // }
}

/// Our `on_set_name` callback: print a warning if the reducer failed.
fn on_name_set(_sender: &Identity, status: &Status, name: &String) {
    if let Status::Failed(err) = status {
        eprintln!("Failed to change name to {:?}: {}", name, err);
    }
}
