use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{sleep, JoinHandle},
    time::Duration,
};

use crate::{
    sensor_outputs::{SensorOutput, SensorOutputs},
    sensors::{
        lock_status_sensor::LockStatusSensor, microphone_usage_sensor::MicrophoneUsageSensor,
        Sensor,
    },
};

const THREAD_SLEEP_DURATION_MS: u64 = 50;

pub enum MainToMonitorMessages {
    SetEguiContext(egui::Context),
    LockStatusSensorInitialized(LockStatusSensor),
}

pub enum MonitorToMainMessages {
    UpdatedSensorOutputs(SensorOutputs),
}

struct SensorMonitor {
    rx_from_main: Receiver<MainToMonitorMessages>,
    tx_to_main: Sender<MonitorToMainMessages>,
    egui_ctx: Option<egui::Context>,

    sensors: Vec<(Box<dyn Sensor>, SensorOutput)>,
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

            sensors: vec![(
                Box::new(MicrophoneUsageSensor::new()),
                SensorOutput::MicrophoneUsage(Default::default()),
            )],
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
                    .send(MonitorToMainMessages::UpdatedSensorOutputs(SensorOutputs {
                        outputs: self.get_sensor_output_snapshot(),
                    }))
                    .unwrap();
                if let Some(ctx) = self.egui_ctx.as_ref() {
                    ctx.request_repaint();
                }
            }

            sleep(Duration::from_millis(THREAD_SLEEP_DURATION_MS));
        }
    }

    fn check_sensor_updates(&mut self) -> bool {
        let initial_sensor_data = self.get_sensor_output_snapshot();
        for (sensor, old_output) in self.sensors.iter_mut() {
            let output = sensor.as_mut().get_output();
            *old_output = output;
        }

        self.get_sensor_output_snapshot()
            .iter()
            .zip(initial_sensor_data.iter())
            .any(|(a, b)| a != b)
    }

    fn process_msg(&mut self, msg: MainToMonitorMessages) -> bool {
        match msg {
            MainToMonitorMessages::SetEguiContext(ctx) => {
                self.egui_ctx = Some(ctx);
            }
            MainToMonitorMessages::LockStatusSensorInitialized(sensor) => {
                self.sensors.push((
                    Box::new(sensor),
                    SensorOutput::LockStatus(Default::default()),
                ));
            }
        }

        false
    }

    fn get_sensor_output_snapshot(&self) -> Vec<SensorOutput> {
        self.sensors
            .iter()
            .map(|(_, output)| output.clone())
            .collect()
    }
}
