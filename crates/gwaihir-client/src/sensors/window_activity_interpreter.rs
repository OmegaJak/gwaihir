use std::{cell::RefCell, rc::Rc};

use gwaihir_client_lib::chrono::{Duration, Utc};
use once_cell::sync::Lazy;

use super::{
    active_window_provider::LockAwareWindowProvider,
    outputs::{sensor_output::SensorOutput, summarized_window_activity::SummarizedWindowActivity},
    window_activity_sensor::WindowActivitySensor,
    Sensor,
};

pub static DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY: Lazy<Duration> =
    Lazy::new(|| Duration::seconds(60 * 10));

pub struct WindowActivityInterpreter {
    window_activity_sensor: WindowActivitySensor<Rc<RefCell<LockAwareWindowProvider>>>,
    lock_aware_active_window_provider: Rc<RefCell<LockAwareWindowProvider>>,
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

    fn updated_sensor_outputs(&mut self, outputs: &[SensorOutput]) {
        for output in outputs {
            if let SensorOutput::LockStatus(lock_status) = output {
                self.lock_aware_active_window_provider
                    .borrow_mut()
                    .currently_locked = lock_status.is_locked();
            }
        }
    }
}

impl WindowActivityInterpreter {
    pub fn new() -> Self {
        let active_window_provider = Rc::new(RefCell::new(LockAwareWindowProvider::new()));
        Self {
            window_activity_sensor: WindowActivitySensor::new(
                *DEFAULT_TIME_TO_KEEP_WINDOW_ACTIVITY,
                active_window_provider.clone(),
            ),
            lock_aware_active_window_provider: active_window_provider,
        }
    }
}
