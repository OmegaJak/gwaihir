use std::collections::VecDeque;

use active_win_pos_rs::get_active_window;
use gwaihir_client_lib::chrono::{Duration, Utc};
use log::error;

use super::outputs::window_activity::{
    ActiveWindow, PreviouslyActiveWindow, SameWindow, WindowActivity,
};

pub struct WindowActivitySensor {
    time_to_keep_activity: Duration,
    current_active_window: Option<ActiveWindow>,
    previously_active_windows: VecDeque<PreviouslyActiveWindow>,
}

impl WindowActivitySensor {
    pub fn new(time_to_keep_activity: Duration) -> Self {
        Self {
            time_to_keep_activity,
            current_active_window: None,
            previously_active_windows: VecDeque::new(),
        }
    }

    pub fn update(&mut self) -> Option<WindowActivity> {
        match get_active_window() {
            Ok(active_window) => {
                let previously_active_window = self.update_currently_active_window(active_window);
                if let Some(window) = previously_active_window {
                    self.previously_active_windows.push_front(window);
                }
            }
            Err(()) => {
                error!("Failed to get the active window");
            }
        }

        self.remove_old_activity();
        self.get_window_activity()
    }

    pub fn get_window_activity(&self) -> Option<WindowActivity> {
        if let Some(current_window) = self.current_active_window.as_ref() {
            Some(WindowActivity {
                current_window: current_window.clone(),
                previously_active_windows: self.previously_active_windows.iter().cloned().collect(),
            })
        } else {
            None
        }
    }

    fn remove_old_activity(&mut self) {
        let cutoff = Utc::now() - self.time_to_keep_activity;
        self.previously_active_windows
            .retain_mut(|w| w.stopped_using > cutoff)
    }

    fn update_currently_active_window(
        &mut self,
        active_window: active_win_pos_rs::ActiveWindow,
    ) -> Option<PreviouslyActiveWindow> {
        if let Some(current_active_window) = self.current_active_window.take() {
            if !current_active_window.same_window_as(&active_window) {
                self.current_active_window = Some(active_window.into());
                return Some(current_active_window.to_no_longer_active());
            } else {
                self.current_active_window = Some(current_active_window);
            }
        } else {
            self.current_active_window = Some(active_window.into());
        }

        None
    }
}
