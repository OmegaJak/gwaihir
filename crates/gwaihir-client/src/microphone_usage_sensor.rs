use gwaihir_client_lib::MicrophoneUsage;
use std::time::{Duration, Instant};
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
use winreg::RegKey;

pub struct MicrophoneUsageSensor {
    last_check_time: Instant,
}

impl MicrophoneUsageSensor {
    pub fn new() -> MicrophoneUsageSensor {
        MicrophoneUsageSensor {
            last_check_time: Instant::now() - Duration::from_millis(500), // Ew
        }
    }

    pub fn check_microphone_usage(&mut self) -> Option<Vec<MicrophoneUsage>> {
        let now = Instant::now();
        if now.duration_since(self.last_check_time) > Duration::from_millis(500) {
            let all_programs_using_microphone = get_all_programs_using_microphone();
            self.last_check_time = now;
            return Some(all_programs_using_microphone);
        }

        None
    }
}

fn get_all_programs_using_microphone() -> Vec<MicrophoneUsage> {
    // If performance becomes a concern, we could maybe usage RegNotifyChangeKeyValue (https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regnotifychangekeyvalue)
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let microphone_packaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone", KEY_READ).unwrap();
    let microphone_nonpackaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\\NonPackaged", KEY_READ).unwrap();
    let in_use: Vec<_> = get_microphone_usage_list(microphone_packaged_store)
        .into_iter()
        .chain(get_microphone_usage_list(microphone_nonpackaged_store).into_iter())
        .filter(|usage| usage.last_used == 0)
        .collect();
    in_use
}

fn get_microphone_usage_list(parent_regkey: winreg::RegKey) -> Vec<MicrophoneUsage> {
    parent_regkey
        .enum_keys()
        .filter_map(|x| x.ok())
        .filter_map(|key| {
            Some(MicrophoneUsage {
                app_name: key.clone(),
                last_used: parent_regkey
                    .open_subkey_with_flags(key, KEY_READ)
                    .ok()?
                    .get_value::<u64, _>("LastUsedTimeStop")
                    .ok()?,
            })
        })
        .collect()
}
