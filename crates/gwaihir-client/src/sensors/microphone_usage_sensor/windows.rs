use crate::sensors::outputs::microphone_usage::{AppName, MicrophoneUsage};
use gwaihir_client_lib::periodic_checker::{HasPeriodicChecker, PeriodicChecker};
use std::path::Path;
use std::time::Duration;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

pub struct WindowsMicrophoneUsageSensor {
    periodic_checker: PeriodicChecker<MicrophoneUsage>,
}

impl HasPeriodicChecker<MicrophoneUsage> for WindowsMicrophoneUsageSensor {
    fn periodic_checker(&self) -> &PeriodicChecker<MicrophoneUsage> {
        &self.periodic_checker
    }

    fn periodic_checker_mut(&mut self) -> &mut PeriodicChecker<MicrophoneUsage> {
        &mut self.periodic_checker
    }
}

impl WindowsMicrophoneUsageSensor {
    pub fn new() -> Self {
        Self {
            periodic_checker: PeriodicChecker::new(
                Box::new(|| MicrophoneUsage {
                    usage: get_all_programs_using_microphone(),
                }),
                Duration::from_millis(500),
            ),
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
    parent_regkey
        .open_subkey_with_flags(app_name, KEY_READ)
        .ok()?
        .get_value::<u64, _>("LastUsedTimeStop")
        .ok()
}

fn prettify(app_key: String) -> String {
    app_key.replace('#', "\\")
}

fn redact_private_info(app_key: String) -> String {
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
