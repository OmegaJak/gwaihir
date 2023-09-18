use std::path::Path;
use std::time::{Duration, Instant};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

use super::outputs::microphone_usage::{AppName, MicrophoneUsage};
use super::outputs::sensor_output::SensorOutput;
use super::Sensor;

pub struct MicrophoneUsageSensor {
    last_check_time: Instant,
    most_recent_data: MicrophoneUsage,
}

impl Sensor for MicrophoneUsageSensor {
    fn get_output(&mut self) -> SensorOutput {
        self.check_microphone_usage();
        SensorOutput::MicrophoneUsage(self.most_recent_data.clone())
    }
}

impl MicrophoneUsageSensor {
    pub fn new() -> Self {
        Self {
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

    let mut in_use = get_apps_using_microphone(microphone_packaged_store);
    in_use.append(&mut get_apps_using_microphone(microphone_nonpackaged_store));
    in_use
}

fn get_apps_using_microphone(parent_regkey: winreg::RegKey) -> Vec<AppName> {
    parent_regkey
        .enum_keys()
        .filter_map(|x| x.ok())
        .filter_map(|key| {
            if get_last_microphone_usage_time(&parent_regkey, &key)? == 0 {
                Some(redact_private_info(prettify(key.clone())).into())
            } else {
                None
            }
        })
        .collect()
}

fn get_last_microphone_usage_time(parent_regkey: &RegKey, app_name: &str) -> Option<u64> {
    Some(
        parent_regkey
            .open_subkey_with_flags(app_name, KEY_READ)
            .ok()?
            .get_value::<u64, _>("LastUsedTimeStop")
            .ok()?,
    )
}

fn prettify(app_key: String) -> String {
    app_key.replace("#", "\\")
}

fn redact_private_info(app_key: String) -> String {
    log::warn!("{}", app_key);
    Path::new(&app_key)
        .file_name()
        .map_or(app_key.clone(), |os_str| {
            os_str.to_string_lossy().to_string()
        })
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn redact_given_windows_path_removes_up_to_filename() {
        let app_key = "C:\\Users\\USERNAME\\appv2\\app.exe";

        let redacted = redact_private_info(app_key.to_string());

        assert_eq!("app.exe".to_string(), redacted);
    }

    #[test]
    pub fn redact_given_app_name_keeps_app_name() {
        let app_key = "windowsstoreapp123";

        let redacted = redact_private_info(app_key.to_string());

        assert_eq!(app_key.to_string(), redacted);
    }
}
