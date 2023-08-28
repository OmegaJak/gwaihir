use crate::sensor_outputs::SensorOutput;

pub mod lock_status_sensor;
pub mod microphone_usage_sensor;

pub trait Sensor {
    fn get_output(&mut self) -> SensorOutput;
}
