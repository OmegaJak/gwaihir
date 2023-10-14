use std::time::Duration;

use gwaihir_client_lib::periodic_checker::{HasPeriodicChecker, PeriodicChecker};
use pulsectl::controllers::{AppControl, SourceController};

use crate::sensors::outputs::microphone_usage::{AppName, MicrophoneUsage};

pub struct LinuxMicrophoneUsageSensor {
    periodic_checker: PeriodicChecker<MicrophoneUsage>,
}

impl HasPeriodicChecker<MicrophoneUsage> for LinuxMicrophoneUsageSensor {
    fn periodic_checker(&self) -> &PeriodicChecker<MicrophoneUsage> {
        &self.periodic_checker
    }

    fn periodic_checker_mut(&mut self) -> &mut PeriodicChecker<MicrophoneUsage> {
        &mut self.periodic_checker
    }
}

impl LinuxMicrophoneUsageSensor {
    pub fn new() -> Self {
        Self {
            periodic_checker: PeriodicChecker::new(
                Box::new(get_mic_usage_closure()),
                Duration::from_millis(500),
            ),
        }
    }
}

fn get_mic_usage_closure() -> impl FnMut() -> MicrophoneUsage {
    let mut mic_data_handler = SourceController::create().unwrap();
    move || {
        let apps = match mic_data_handler.list_applications() {
            Ok(apps) => apps
                .into_iter()
                .filter_map(|app| {
                    app.proplist
                        .get_str("application.process.binary")
                        .map(AppName::new)
                })
                .collect(),
            Err(e) => {
                log::error!("Failed to get microphone usage: {}", e);
                vec![]
            }
        };
        MicrophoneUsage { usage: apps }
    }
}
