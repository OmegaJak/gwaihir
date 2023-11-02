use super::{
    active_window_provider::{ActiveWindowProvider, MaybeLocked, RawActiveWindow},
    window_title_mapper::WindowTitleMappings,
};
use crate::{
    sensor_monitor_thread::MonitorToMainMessages,
    sensors::outputs::window_activity::{
        ActiveWindow, PreviouslyActiveWindow, WindowActivity, WindowExtensions, WindowName,
    },
};
use gwaihir_client_lib::chrono::{Duration, Utc};
use log::error;
use std::{
    collections::VecDeque,
    sync::{mpsc::Sender, Arc, RwLock},
};

pub struct WindowActivitySensor<T> {
    time_to_keep_activity: Duration,
    current_active_window: Option<ActiveWindow>,
    previously_active_windows: VecDeque<PreviouslyActiveWindow>,
    active_window_provider: T,
    tx_to_main: Sender<MonitorToMainMessages>,
    window_title_mappings: Arc<RwLock<WindowTitleMappings>>,
}

impl<T> WindowActivitySensor<T>
where
    T: ActiveWindowProvider,
{
    pub fn new(
        time_to_keep_activity: Duration,
        active_window_provider: T,
        tx_to_main: Sender<MonitorToMainMessages>,
        window_title_mappings: Arc<RwLock<WindowTitleMappings>>,
    ) -> Self {
        Self {
            time_to_keep_activity,
            current_active_window: None,
            previously_active_windows: VecDeque::new(),
            active_window_provider,
            tx_to_main,
            window_title_mappings,
        }
    }

    pub fn update(&mut self) -> Option<WindowActivity> {
        match self.active_window_provider.get_active_window() {
            Ok(active_window) => {
                self.record_observed_active_window(&active_window);
                let active_window = self.map_raw_window(active_window);
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
        self.current_active_window
            .as_ref()
            .map(|current_window| WindowActivity {
                current_window: current_window.clone(),
                previously_active_windows: self.previously_active_windows.iter().cloned().collect(),
            })
    }

    fn remove_old_activity(&mut self) {
        let cutoff = Utc::now() - self.time_to_keep_activity;
        self.previously_active_windows
            .retain_mut(|w| w.stopped_using > cutoff)
    }

    fn update_currently_active_window(
        &mut self,
        new_active_window: ActiveWindow,
    ) -> Option<PreviouslyActiveWindow> {
        if let Some(current_active_window) = self.current_active_window.take() {
            if !current_active_window.same_window_as(&new_active_window) {
                self.current_active_window = Some(new_active_window);
                return Some(current_active_window.into_no_longer_active());
            } else {
                self.current_active_window = Some(current_active_window);
            }
        } else {
            self.current_active_window = Some(new_active_window);
        }

        None
    }

    fn record_observed_active_window(&mut self, active_window: &RawActiveWindow) {
        if let MaybeLocked::Normal(window_data) = &active_window.window_data {
            self.tx_to_main
                .send(MonitorToMainMessages::ObservedWindowTitle(
                    window_data.identifiers.clone(),
                ))
                .unwrap();
        }
    }

    fn map_raw_window(&self, raw_active_window: RawActiveWindow) -> ActiveWindow {
        let window_name = match raw_active_window.window_data {
            MaybeLocked::Locked => WindowName::Locked,
            MaybeLocked::Normal(window_data) => {
                let mapped_title = self
                    .window_title_mappings
                    .read()
                    .unwrap()
                    .map(window_data.identifiers);
                WindowName::Normal(mapped_title)
            }
        };
        ActiveWindow {
            window_name,
            started_using: Utc::now(),
        }
    }
}
