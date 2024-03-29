use super::{
    backoff_executor::BackoffExecutor, offline_network_interface::OfflineNetworkInterface,
};
use crate::{
    networking::backoff_executor::BackoffExecutionAction,
    sensors::outputs::sensor_outputs::SensorOutputs,
};
use delegate::delegate;
use gwaihir_client_lib::{NetworkInterface, NetworkInterfaceCreator, NetworkType, RemoteUpdate};
use log::{info, warn};
use networking_spacetimedb::{SpacetimeDBCreationParameters, SpacetimeDBInterface};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

const MIN_TIME_BETWEEN_RECONNECT_ATTEMPTS: Duration = Duration::from_secs(1);
const MAX_TIME_BETWEEN_RECONNECT_ATTEMPTS: Duration = Duration::from_secs(60);

pub struct NetworkManager {
    network: Box<dyn NetworkInterface<SensorOutputs>>,
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    network_rx: Receiver<RemoteUpdate<SensorOutputs>>,
    egui_ctx: egui::Context,
    backoff: BackoffExecutor,
}

impl NetworkManager {
    pub fn new<N, P>(egui_ctx: egui::Context, network_creation_parameters: P) -> Self
    where
        N: NetworkInterface<SensorOutputs>
            + NetworkInterfaceCreator<SensorOutputs, N, P>
            + Send
            + 'static,
        P: Send + 'static,
    {
        let (network_tx, network_rx) = mpsc::channel();
        let network = try_init_network_interface::<N, P>(
            network_tx.clone(),
            egui_ctx.clone(),
            network_creation_parameters,
        );
        Self {
            network,
            network_tx,
            network_rx,
            egui_ctx: egui_ctx.clone(),
            backoff: BackoffExecutor::new(
                MIN_TIME_BETWEEN_RECONNECT_ATTEMPTS,
                MAX_TIME_BETWEEN_RECONNECT_ATTEMPTS,
            ),
        }
    }

    pub fn try_recv(&mut self) -> Result<RemoteUpdate<SensorOutputs>, mpsc::TryRecvError> {
        self.network_rx.try_recv()
    }

    pub fn queue_fake_update(
        &mut self,
        update: RemoteUpdate<SensorOutputs>,
    ) -> Result<(), mpsc::SendError<RemoteUpdate<SensorOutputs>>> {
        self.network_tx.send(update)
    }

    pub fn try_reconnect_if_needed(&mut self) {
        if self.is_offline() {
            self.backoff.maybe_execute(
                || {
                    let success = self.network.try_reconnect();
                    info!("Attempting reconnection to the network");
                    if success {
                        info!("Successfully reconnected to the network");
                        BackoffExecutionAction::Reset
                    } else {
                        warn!("Failed to reconnect to the network, trying again soon");
                        BackoffExecutionAction::KeepTrying
                    }
                },
                Instant::now(),
            );
        }
    }

    pub fn reinit_network(&mut self, new_network_type: NetworkType, spacetimedb_db_name: String) {
        let network_tx = self.network_tx.clone();
        let egui_ctx = self.egui_ctx.clone();
        self.network.disconnect();
        match new_network_type {
            NetworkType::Offline => self.network = get_offline_network(network_tx, egui_ctx),
            NetworkType::SpacetimeDB => {
                let creation_params = SpacetimeDBCreationParameters {
                    db_name: spacetimedb_db_name,
                };
                self.network = try_init_network_interface::<SpacetimeDBInterface, _>(
                    network_tx,
                    egui_ctx,
                    creation_params,
                )
            }
        }
    }

    pub fn is_offline(&self) -> bool {
        !self.network.is_connected()
    }
}

impl NetworkInterface<SensorOutputs> for NetworkManager {
    delegate! {
        to self.network {
            fn publish_update(&self, sensor_outputs: SensorOutputs);
            fn set_username(&self, name: String);
            fn get_current_user_id(&self) -> Option<gwaihir_client_lib::UniqueUserId>;
            fn get_network_type(&self) -> gwaihir_client_lib::NetworkType;
            fn is_connected(&self) -> bool;
            fn try_reconnect(&mut self) -> bool;
            fn disconnect(&mut self);
        }
    }
}

fn try_init_network_interface<N, P>(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    egui_ctx: egui::Context,
    creation_parameters: P,
) -> Box<dyn NetworkInterface<SensorOutputs> + Send>
where
    N: NetworkInterface<SensorOutputs>
        + NetworkInterfaceCreator<SensorOutputs, N, P>
        + Send
        + 'static,
    P: Send + 'static,
{
    let network_tx_clone = network_tx.clone();
    let ctx_clone = egui_ctx.clone();
    run_with_timeout(
        move || {
            Box::new(N::create(
                get_remote_update_callback(network_tx.clone(), egui_ctx.clone()),
                get_on_disconnect_callback(egui_ctx),
                creation_parameters,
            )) as Box<dyn NetworkInterface<SensorOutputs> + Send>
        },
        Duration::from_secs(5),
        Some(std::thread::Builder::new().name("network_interface_initializer".to_string())),
    )
    .unwrap_or_else(move |_e| {
        warn!(
            "Defaulting to offline network interface because initialization of the primary failed"
        );
        get_offline_network(network_tx_clone, ctx_clone)
    })
}

fn get_offline_network(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    egui_ctx: egui::Context,
) -> Box<OfflineNetworkInterface<SensorOutputs>> {
    Box::new(OfflineNetworkInterface::create(
        get_remote_update_callback(network_tx.clone(), egui_ctx.clone()),
        get_on_disconnect_callback(egui_ctx),
        (),
    ))
}

fn get_remote_update_callback(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    ctx_clone: egui::Context,
) -> impl Fn(RemoteUpdate<SensorOutputs>) + Clone {
    move |update| {
        network_tx.send(update).unwrap();
        ctx_clone.request_repaint();
    }
}

fn get_on_disconnect_callback(ctx_clone: egui::Context) -> impl FnMut() {
    move || {
        ctx_clone.request_repaint();
    }
}

// https://stackoverflow.com/a/74234262/6581675
fn run_with_timeout<F, T>(
    f: F,
    timeout: Duration,
    thread_builder: Option<std::thread::Builder>,
) -> Result<T, ()>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let thread_builder = thread_builder.unwrap_or_else(std::thread::Builder::new);
    let handle = thread_builder
        .spawn(move || {
            let result = f();
            tx.send(result).ok(); // either it went fine, or our handle was released because we took too long
        })
        .unwrap();

    match rx.recv_timeout(timeout) {
        Ok(result) => Ok(result),
        Err(mpsc::RecvTimeoutError::Timeout) => Err(()),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // thread crashed
            if handle.is_finished() {
                let _ = handle.join(); // print crash msg
            } else {
                unreachable!(
                    "it shouldn't be possible for the thread to drop the sender without crashing"
                )
            }
            Err(())
        }
    }
}
