use crate::{
    offline_network_interface::OfflineNetworkInterface,
    sensors::outputs::sensor_outputs::SensorOutputs,
};
use delegate::delegate;
use gwaihir_client_lib::{NetworkInterface, NetworkInterfaceCreator, NetworkType, RemoteUpdate};
use log::warn;
use networking_spacetimedb::SpacetimeDBInterface;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

pub struct NetworkManager {
    network: Box<dyn NetworkInterface<SensorOutputs>>,
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    network_rx: Receiver<RemoteUpdate<SensorOutputs>>,
    egui_ctx: egui::Context,
}

impl NetworkManager {
    pub fn new<N>(egui_ctx: egui::Context) -> Self
    where
        N: NetworkInterface<SensorOutputs>
            + NetworkInterfaceCreator<SensorOutputs, N>
            + Send
            + 'static,
    {
        let (network_tx, network_rx) = mpsc::channel();
        let network = try_init_network_interface::<N>(network_tx.clone(), egui_ctx.clone());
        Self {
            network,
            network_tx,
            network_rx,
            egui_ctx: egui_ctx.clone(),
        }
    }

    pub fn try_recv(&self) -> Result<RemoteUpdate<SensorOutputs>, mpsc::TryRecvError> {
        self.network_rx.try_recv()
    }

    pub fn reinit_network(&mut self, new_network_type: NetworkType) {
        let network_tx = self.network_tx.clone();
        let egui_ctx = self.egui_ctx.clone();
        match new_network_type {
            NetworkType::Offline => self.network = get_offline_network(network_tx, egui_ctx),
            NetworkType::SpacetimeDB => {
                self.network =
                    try_init_network_interface::<SpacetimeDBInterface>(network_tx, egui_ctx)
            }
        }
    }

    pub fn is_offline(&self) -> bool {
        self.network.get_network_type() == NetworkType::Offline
    }
}

impl NetworkInterface<SensorOutputs> for NetworkManager {
    delegate! {
        to self.network {
            fn publish_update(&self, sensor_outputs: SensorOutputs);
            fn set_username(&self, name: String);
            fn get_current_user_id(&self) -> Option<gwaihir_client_lib::UniqueUserId>;
            fn get_network_type(&self) -> gwaihir_client_lib::NetworkType;
        }
    }
}

fn try_init_network_interface<N>(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    egui_ctx: egui::Context,
) -> Box<dyn NetworkInterface<SensorOutputs> + Send>
where
    N: NetworkInterface<SensorOutputs> + NetworkInterfaceCreator<SensorOutputs, N> + Send + 'static,
{
    let network_tx_clone = network_tx.clone();
    let ctx_clone = egui_ctx.clone();
    run_with_timeout(
        move || {
            Box::new(N::new(
                get_remote_update_callback(network_tx.clone(), egui_ctx.clone()),
                get_on_disconnect_callback(network_tx, egui_ctx),
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
    Box::new(OfflineNetworkInterface::new(
        get_remote_update_callback(network_tx.clone(), egui_ctx.clone()),
        get_on_disconnect_callback(network_tx, egui_ctx),
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

fn get_on_disconnect_callback(
    network_tx: Sender<RemoteUpdate<SensorOutputs>>,
    ctx_clone: egui::Context,
) -> impl FnOnce() {
    move || {
        network_tx.send(RemoteUpdate::Disconnected).unwrap();
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
    let thread_builder = thread_builder.unwrap_or_else(|| std::thread::Builder::new());
    let handle = thread_builder
        .spawn(move || {
            let result = f();
            match tx.send(result) {
                Ok(()) => {} // everything good
                Err(_) => {} // we have been released, don't panic
            }
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
