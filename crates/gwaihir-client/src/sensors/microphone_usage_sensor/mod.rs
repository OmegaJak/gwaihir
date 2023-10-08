use super::{
    outputs::{microphone_usage::MicrophoneUsage, sensor_output::SensorOutput},
    Sensor,
};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::WindowsMicrophoneUsageSensor;

pub trait MicrophoneUsageSensor {
    fn check_microphone_usage(&mut self);
    fn most_recent_data(&self) -> MicrophoneUsage;
}

impl<T> Sensor for T
where
    T: MicrophoneUsageSensor,
{
    fn get_output(&mut self) -> SensorOutput {
        self.check_microphone_usage();
        SensorOutput::MicrophoneUsage(self.most_recent_data().clone())
    }
}

#[allow(unreachable_code)]
pub fn try_get_sensor() -> Option<Box<dyn Sensor>> {
    #[cfg(target_os = "windows")]
    return Some(Box::new(WindowsMicrophoneUsageSensor::new()));

    None::<Box<dyn Sensor>>
}
