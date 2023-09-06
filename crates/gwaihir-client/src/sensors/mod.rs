use crate::sensors::outputs::sensor_output::SensorOutput;

pub mod lock_status_sensor;
pub mod microphone_usage_sensor;
pub mod outputs;
pub mod window_activity_sensor;

pub trait Sensor {
    fn get_output(&mut self) -> SensorOutput;
}
