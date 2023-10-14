use super::{
    outputs::{microphone_usage::MicrophoneUsage, sensor_output::SensorOutput},
    Sensor,
};
use gwaihir_client_lib::periodic_checker::HasPeriodicChecker;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows::WindowsMicrophoneUsageSensor;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux::LinuxMicrophoneUsageSensor;

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

impl<T> MicrophoneUsageSensor for T
where
    T: HasPeriodicChecker<MicrophoneUsage>,
{
    fn check_microphone_usage(&mut self) {
        self.periodic_checker_mut().check();
    }

    fn most_recent_data(&self) -> MicrophoneUsage {
        self.periodic_checker().last_check_result()
    }
}

#[allow(unreachable_code)]
pub fn try_get_sensor() -> Option<Box<dyn Sensor>> {
    #[cfg(target_os = "windows")]
    return Some(Box::new(WindowsMicrophoneUsageSensor::new()));

    #[cfg(target_os = "linux")]
    return Some(Box::new(LinuxMicrophoneUsageSensor::new()));

    None::<Box<dyn Sensor>>
}
