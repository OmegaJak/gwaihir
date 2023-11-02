use gwaihir_client_lib::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const LOCK_SCREEN_WINDOW_NAME: &str = "Locked";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct WindowActivity {
    pub current_window: ActiveWindow,
    pub previously_active_windows: Vec<PreviouslyActiveWindow>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ActiveWindow {
    pub window_name: WindowName,
    pub started_using: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PreviouslyActiveWindow {
    pub window_name: WindowName,
    pub started_using: DateTime<Utc>,
    pub stopped_using: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum WindowName {
    Locked,
    Normal(String),
}

impl From<WindowName> for String {
    fn from(value: WindowName) -> Self {
        match value {
            WindowName::Locked => LOCK_SCREEN_WINDOW_NAME.to_string(),
            WindowName::Normal(name) => name,
        }
    }
}

impl std::fmt::Display for WindowName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

pub trait RepresentsWindow {
    fn window_name(&self) -> WindowName;
}

pub trait WindowExtensions {
    fn is_lock_window(&self) -> bool;
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool;
}

impl RepresentsWindow for ActiveWindow {
    fn window_name(&self) -> WindowName {
        self.window_name.clone()
    }
}

impl ActiveWindow {
    pub fn into_no_longer_active(self) -> PreviouslyActiveWindow {
        PreviouslyActiveWindow {
            window_name: self.window_name,
            started_using: self.started_using,
            stopped_using: Utc::now(),
        }
    }
}

impl<T: RepresentsWindow> WindowExtensions for T {
    fn same_window_as(&self, other: &impl RepresentsWindow) -> bool {
        self.window_name() == other.window_name()
    }

    fn is_lock_window(&self) -> bool {
        matches!(self.window_name(), WindowName::Locked)
    }
}
