use crate::sensors::outputs::sensor_output::SensorOutput;

pub mod keyboard_mouse_event_provider;
pub mod keyboard_mouse_sensor;
pub mod lock_status_sensor;
pub mod microphone_usage_sensor;
pub mod outputs;
pub mod window_sensor;

pub trait Sensor {
    fn get_output(&mut self) -> SensorOutput;
    fn updated_sensor_outputs(&mut self, _outputs: &[SensorOutput]) {}
}
