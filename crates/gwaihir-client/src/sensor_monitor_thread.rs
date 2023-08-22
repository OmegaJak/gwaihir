use std::{
    borrow::BorrowMut,
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{sleep, JoinHandle},
    time::Duration,
};

use gwaihir_client_lib::SensorData;

use crate::{
    lock_status_sensor::{LockStatusSensor, SessionEvent},
    microphone_usage_sensor::MicrophoneUsageSensor,
};

const THREAD_SLEEP_DURATION_MS: u64 = 50;

pub enum MainToMonitorMessages {
    SetEguiContext(egui::Context),
    LockStatusSensorInitialized(LockStatusSensor),
}

pub enum MonitorToMainMessages {
    UpdatedSensorData(SensorData),
}

struct SensorMonitor {
    rx_from_main: Receiver<MainToMonitorMessages>,
    tx_to_main: Sender<MonitorToMainMessages>,
    egui_ctx: Option<egui::Context>,

    lock_status_sensor: Option<LockStatusSensor>,
    microphone_usage_sensor: MicrophoneUsageSensor,

    sensor_data: SensorData,
}

pub fn create_sensor_monitor_thread() -> (
    JoinHandle<()>,
    Sender<MainToMonitorMessages>,
    Receiver<MonitorToMainMessages>,
) {
    let (main_tx, monitor_rx) = channel();
    let (monitor_tx, main_rx) = channel();
    let handle = std::thread::spawn(|| {
        let mut monitor = SensorMonitor {
            rx_from_main: monitor_rx,
            tx_to_main: monitor_tx,
            egui_ctx: None,
            lock_status_sensor: None,
            microphone_usage_sensor: MicrophoneUsageSensor::new(),

            sensor_data: SensorData::default(),
        };
        monitor.run();
    });

    (handle, main_tx, main_rx)
}

impl SensorMonitor {
    fn run(&mut self) {
        loop {
            match self.rx_from_main.try_recv() {
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    return;
                }
                Ok(msg) => {
                    self.process_msg(msg);
                }
            }

            if self.check_sensor_updates() {
                self.tx_to_main
                    .send(MonitorToMainMessages::UpdatedSensorData(
                        self.sensor_data.clone(),
                    ))
                    .unwrap();
                if let Some(ctx) = self.egui_ctx.take() {
                    ctx.request_repaint();
                    self.egui_ctx = Some(ctx);
                }
            }

            sleep(Duration::from_millis(THREAD_SLEEP_DURATION_MS));
        }
    }

    fn check_sensor_updates(&mut self) -> bool {
        let initial_sensor_data = self.sensor_data.clone();
        if let Some(mut sensor) = self.lock_status_sensor.take() {
            match sensor.recv() {
                Some(SessionEvent::Locked) => {
                    self.sensor_data.num_locks += 1;
                }
                Some(SessionEvent::Unlocked) => {
                    self.sensor_data.num_unlocks += 1;
                }
                None => (),
            }

            self.lock_status_sensor = Some(sensor);
        }

        let microphone_usage = self.microphone_usage_sensor.check_microphone_usage();
        if let Some(usage) = microphone_usage {
            self.sensor_data.microphone_usage = usage;
        }

        self.sensor_data != initial_sensor_data
    }

    fn process_msg(&mut self, msg: MainToMonitorMessages) -> bool {
        match msg {
            MainToMonitorMessages::SetEguiContext(ctx) => {
                self.egui_ctx = Some(ctx);
            }
            MainToMonitorMessages::LockStatusSensorInitialized(sensor) => {
                self.lock_status_sensor = Some(sensor);
            }
        }

        false
    }
}
