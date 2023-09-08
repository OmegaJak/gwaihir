use gwaihir_client_lib::chrono::{Duration, Utc};
use once_cell::sync::Lazy;

use super::{
    outputs::{sensor_output::SensorOutput, summarized_window_activity::SummarizedWindowActivity},
    window_activity_sensor::WindowActivitySensor,
    Sensor,
};

pub const DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY: Lazy<Duration> =
    Lazy::new(|| Duration::seconds(60 * 10));

pub struct WindowActivityInterpreter {
    window_activity_sensor: WindowActivitySensor,
}

impl Sensor for WindowActivityInterpreter {
    fn get_output(&mut self) -> SensorOutput {
        if let Some(window_activity) = self.window_activity_sensor.update() {
            let now = Utc::now();
            let cutoff = now - *DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY;
            SummarizedWindowActivity::summarize(&window_activity, now, cutoff).into()
        } else {
            SensorOutput::Empty
        }
    }
}

impl WindowActivityInterpreter {
    pub fn new() -> Self {
        Self {
            window_activity_sensor: WindowActivitySensor::new(
                *DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY,
            ),
        }
    }
}
