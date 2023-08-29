use std::time::{Duration, Instant};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

use crate::sensor_outputs::microphone_usage::{AppName, MicrophoneUsage};
use crate::sensor_outputs::SensorOutput;

use super::Sensor;

pub struct MicrophoneUsageSensor {
    last_check_time: Instant,
    most_recent_data: MicrophoneUsage,
}

struct AppMicrophoneUsage {
    app_name: AppName,
    last_used: u64,
}

impl Sensor for MicrophoneUsageSensor {
    fn get_output(&mut self) -> SensorOutput {
        self.check_microphone_usage();
        SensorOutput::MicrophoneUsage(self.most_recent_data.clone())
    }
}

impl MicrophoneUsageSensor {
    pub fn new() -> MicrophoneUsageSensor {
        MicrophoneUsageSensor {
            last_check_time: Instant::now() - Duration::from_millis(500), // Ew
            most_recent_data: Default::default(),
        }
    }

    pub fn check_microphone_usage(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_check_time) > Duration::from_millis(500) {
            let all_programs_using_microphone = get_all_programs_using_microphone();
            self.last_check_time = now;
            self.most_recent_data = MicrophoneUsage {
                usage: all_programs_using_microphone,
            }
        }
    }
}

fn get_all_programs_using_microphone() -> Vec<AppName> {
    // If performance becomes a concern, we could maybe usage RegNotifyChangeKeyValue (https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regnotifychangekeyvalue)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let microphone_packaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone", KEY_READ).unwrap();
    let microphone_nonpackaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\\NonPackaged", KEY_READ).unwrap();
    let in_use: Vec<_> = get_microphone_usage_list(microphone_packaged_store)
        .into_iter()
        .chain(get_microphone_usage_list(microphone_nonpackaged_store).into_iter())
        .filter(|usage| usage.last_used == 0)
        .map(|a| a.app_name)
        .collect();
    in_use
}

fn get_microphone_usage_list(parent_regkey: winreg::RegKey) -> Vec<AppMicrophoneUsage> {
    parent_regkey
        .enum_keys()
        .filter_map(|x| x.ok())
        .filter_map(|key| {
            Some(AppMicrophoneUsage {
                app_name: key.clone().into(),
                last_used: parent_regkey
                    .open_subkey_with_flags(key, KEY_READ)
                    .ok()?
                    .get_value::<u64, _>("LastUsedTimeStop")
                    .ok()?,
            })
        })
        .collect()
}
