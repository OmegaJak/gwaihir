use active_win_pos_rs::get_active_window;
use bounded_vec_deque::BoundedVecDeque;
use log::error;

use super::{
    outputs::{
        sensor_output::SensorOutput,
        window_activity::{ActiveWindow, PreviouslyActiveWindow, SameWindow, WindowActivity},
    },
    Sensor,
};

const ACTIVE_WINDOW_HISTORY_LENGTH: usize = 7;

pub struct WindowActivitySensor {
    current_active_window: Option<ActiveWindow>,
    previously_active_windows: BoundedVecDeque<PreviouslyActiveWindow>,
}

impl Sensor for WindowActivitySensor {
    fn get_output(&mut self) -> super::outputs::sensor_output::SensorOutput {
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

        self.get_window_activity()
    }
}

impl WindowActivitySensor {
    pub fn new() -> Self {
        Self {
            current_active_window: None,
            previously_active_windows: BoundedVecDeque::new(ACTIVE_WINDOW_HISTORY_LENGTH),
        }
    }

    fn get_window_activity(&self) -> SensorOutput {
        if let Some(current_window) = self.current_active_window.as_ref() {
            SensorOutput::WindowActivity(WindowActivity {
                current_window: current_window.clone(),
                previously_active_windows: self.previously_active_windows.iter().cloned().collect(),
            })
        } else {
            SensorOutput::Empty
        }
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
