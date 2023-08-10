use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{sleep, JoinHandle},
    time::Duration,
};

use crate::{
    lock_status_sensor::{LockStatusSensor, SessionEvent},
    microphone_usage_sensor::MicrophoneUsageSensor,
};

const THREAD_SLEEP_DURATION_MS: u64 = 50;

pub enum MainToMonitorMessages {
    SetEguiContext(egui::Context),
    LockStatusSensorInitialized(LockStatusSensor),
}

struct SensorMonitor {
    rx_from_main: Receiver<MainToMonitorMessages>,
    egui_ctx: Option<egui::Context>,
    lock_status_sensor: Option<LockStatusSensor>,
    microphone_usage_sensor: MicrophoneUsageSensor,
}

pub fn create_sensor_monitor_thread() -> (JoinHandle<()>, Sender<MainToMonitorMessages>) {
    let (main_tx, monitor_rx) = channel();
    let handle = std::thread::spawn(|| {
        let mut monitor = SensorMonitor {
            rx_from_main: monitor_rx,
            egui_ctx: None,
            lock_status_sensor: None,
            microphone_usage_sensor: MicrophoneUsageSensor::new(),
        };
        monitor.run();
    });

    (handle, main_tx)
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

            self.check_sensors();

            sleep(Duration::from_millis(THREAD_SLEEP_DURATION_MS));
        }
    }

    fn check_sensors(&mut self) {
        if let Some(mut sensor) = self.lock_status_sensor.take() {
            match sensor.recv() {
                Some(SessionEvent::Locked) => {
                    println!("Locked!!");
                }
                Some(SessionEvent::Unlocked) => {
                    println!("Unlocked!!");
                }
                None => (),
            }

            self.lock_status_sensor = Some(sensor);
        }

        self.microphone_usage_sensor.check_microphone_usage();
    }

    fn process_msg(&mut self, msg: MainToMonitorMessages) -> bool {
        match msg {
            // MainToMonitorMessages::Shutdown => {
            //     return true;
            // }
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
