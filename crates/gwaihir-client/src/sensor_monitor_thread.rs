use std::{
    sync::mpsc::{channel, Receiver, Sender, TryRecvError},
    thread::{sleep, JoinHandle},
    time::Duration,
};

use crate::sensors::{
    microphone_usage_sensor::MicrophoneUsageSensor,
    outputs::{sensor_output::SensorOutput, sensor_outputs::SensorOutputs},
    Sensor,
};

use crate::sensors::lock_status_sensor::LockStatusSensor;

const THREAD_SLEEP_DURATION_MS: u64 = 50;

pub enum MainToMonitorMessages {
    SetEguiContext(egui::Context),
    LockStatusSensorInitialized(LockStatusSensor),
}

#[derive(Debug)]
pub enum MonitorToMainMessages {
    UpdatedSensorOutputs(SensorOutputs),
}

struct SensorMonitor {
    rx_from_main: Receiver<MainToMonitorMessages>,
    tx_to_main: Sender<MonitorToMainMessages>,
    egui_ctx: Option<egui::Context>,

    sensors: Vec<(Box<dyn Sensor>, SensorOutput)>,
    last_sent_outputs: Vec<SensorOutput>,
}

pub fn create_sensor_monitor_thread() -> (
    JoinHandle<()>,
    Sender<MainToMonitorMessages>,
    Receiver<MonitorToMainMessages>,
) {
    let (main_tx, monitor_rx) = channel();
    let (monitor_tx, main_rx) = channel();
    let handle = std::thread::spawn(|| {
        let mut monitor = SensorMonitor::new(monitor_rx, monitor_tx);
        monitor.run();
    });

    (handle, main_tx, main_rx)
}

impl SensorMonitor {
    fn new(
        rx_from_main: Receiver<MainToMonitorMessages>,
        tx_to_main: Sender<MonitorToMainMessages>,
    ) -> Self {
        SensorMonitor {
            rx_from_main,
            tx_to_main,
            egui_ctx: None,

            sensors: vec![(
                Box::new(MicrophoneUsageSensor::new()),
                SensorOutput::MicrophoneUsage(Default::default()),
            )],
            last_sent_outputs: Vec::new(),
        }
    }

    fn run(&mut self) {
        loop {
            self.loop_body();

            sleep(Duration::from_millis(THREAD_SLEEP_DURATION_MS));
        }
    }

    fn loop_body(&mut self) {
        self.receive_msgs_from_main();
        self.send_sensor_msgs_to_main();
    }

    fn receive_msgs_from_main(&mut self) {
        match self.rx_from_main.try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                panic!("Communication with main unexpectedly disconnected");
            }
            Ok(msg) => {
                self.process_msg(msg);
            }
        }
    }

    fn send_sensor_msgs_to_main(&mut self) {
        if self.check_sensor_updates() {
            self.tx_to_main
                .send(MonitorToMainMessages::UpdatedSensorOutputs(SensorOutputs {
                    outputs: self.get_sensor_output_snapshot(),
                }))
                .unwrap();
            self.last_sent_outputs = self.get_sensor_output_snapshot();
            if let Some(ctx) = self.egui_ctx.as_ref() {
                ctx.request_repaint();
            }
        }
    }

    fn check_sensor_updates(&mut self) -> bool {
        for (sensor, old_output) in self.sensors.iter_mut() {
            let updated_output = sensor.as_mut().get_output();
            *old_output = updated_output;
        }

        self.get_sensor_output_snapshot() != self.last_sent_outputs
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

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    struct MonitorAndChannels {
        monitor: SensorMonitor,
        main_to_monitor_tx: Sender<MainToMonitorMessages>,
        monitor_to_main_rx: Receiver<MonitorToMainMessages>,
    }

    fn init_monitor() -> MonitorAndChannels {
        let (main_to_monitor_tx, main_to_monitor_rx) = channel();
        let (monitor_to_main_tx, monitor_to_main_rx) = channel();
        let monitor = SensorMonitor::new(main_to_monitor_rx, monitor_to_main_tx);
        MonitorAndChannels {
            monitor,
            main_to_monitor_tx,
            monitor_to_main_rx,
        }
    }

    fn init_monitor_and_flush_initial_messages() -> MonitorAndChannels {
        let mut monitor_and_channels = init_monitor();
        monitor_and_channels.monitor.loop_body();
        while let Ok(_) = monitor_and_channels.monitor_to_main_rx.try_recv() {}
        monitor_and_channels
    }

    #[test]
    fn sends_one_sensor_update_on_startup() {
        let mut mc = init_monitor();

        mc.monitor.loop_body();

        assert_matches!(
            mc.monitor_to_main_rx.try_recv(),
            Ok(MonitorToMainMessages::UpdatedSensorOutputs(_))
        );
        assert_matches!(mc.monitor_to_main_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn sends_no_sensor_update_on_second_loop_with_no_sensor_changes() {
        let mut mc = init_monitor_and_flush_initial_messages();

        mc.monitor.loop_body();

        assert_matches!(mc.monitor_to_main_rx.try_recv(), Err(TryRecvError::Empty));
    }

    #[test]
    fn sends_lock_status_update_once_sensor_is_ready() {
        let mut mc = init_monitor_and_flush_initial_messages();
        let (lock_status_sensor, _session_event_tx) = LockStatusSensor::new();

        mc.main_to_monitor_tx
            .send(MainToMonitorMessages::LockStatusSensorInitialized(
                lock_status_sensor,
            ))
            .unwrap();

        mc.monitor.loop_body();
        assert_matches!(
            mc.monitor_to_main_rx.try_recv(),
            Ok(MonitorToMainMessages::UpdatedSensorOutputs(_))
        );
    }
}
