use std::time::{Duration, Instant, SystemTime};
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

    pub fn check_microphone_usage(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_check_time) > Duration::from_millis(500) {
            print_programs_using_microphone();
            self.last_check_time = now;
        }
    }
}

fn print_programs_using_microphone() {
    let in_use = get_all_programs_last_microphone_usage();

    if in_use.is_empty() {
        println!("No devices listening to microphone");
    } else {
        for (key, last_access) in in_use.iter() {
            if *last_access == 0 {
                println!("Currently in use: {}", key);
            }
        }
    }
}

fn get_all_programs_last_microphone_usage() -> Vec<(String, u64)> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let microphone_packaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone", KEY_READ).unwrap();
    let microphone_nonpackaged_store = hkcu.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\\NonPackaged", KEY_READ).unwrap();
    let in_use: Vec<_> = get_microphone_usage_list(microphone_packaged_store)
        .into_iter()
        .chain(get_microphone_usage_list(microphone_nonpackaged_store).into_iter())
        .collect();
    in_use
}

fn get_microphone_usage_list(parent_regkey: winreg::RegKey) -> Vec<(String, u64)> {
    parent_regkey
        .enum_keys()
        .filter_map(|x| x.ok())
        .filter_map(|key| {
            Some((
                key.clone(),
                parent_regkey
                    .open_subkey_with_flags(key, KEY_READ)
                    .ok()?
                    .get_value::<u64, _>("LastUsedTimeStop")
                    .ok()?,
            ))
        })
        .collect()
}
