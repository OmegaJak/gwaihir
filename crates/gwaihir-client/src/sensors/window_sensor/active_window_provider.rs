use crate::sensors::outputs::window_activity::{RepresentsWindow, WindowName};
use std::{cell::RefCell, rc::Rc};

pub enum MaybeLocked<T> {
    Locked,
    Normal(T),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WindowIdentifiers {
    pub app_name: String,
    pub window_title: String,
}

pub struct RawWindowData {
    pub identifiers: WindowIdentifiers,
}

pub struct RawActiveWindow {
    pub window_data: MaybeLocked<RawWindowData>,
}

pub trait ActiveWindowProvider {
    fn get_active_window(&self) -> Result<RawActiveWindow, ()>;
}

pub struct LockAwareWindowProvider {
    pub currently_locked: bool,
}

impl LockAwareWindowProvider {
    pub fn new() -> Self {
        LockAwareWindowProvider {
            currently_locked: false,
        }
    }
}

impl ActiveWindowProvider for LockAwareWindowProvider {
    fn get_active_window(&self) -> Result<RawActiveWindow, ()> {
        if self.currently_locked {
            Ok(RawActiveWindow {
                window_data: MaybeLocked::Locked,
            })
        } else {
            active_win_pos_rs::get_active_window().map(|w| w.into())
        }
    }
}

impl ActiveWindowProvider for Rc<RefCell<LockAwareWindowProvider>> {
    fn get_active_window(&self) -> Result<RawActiveWindow, ()> {
        self.borrow().get_active_window()
    }
}

impl From<active_win_pos_rs::ActiveWindow> for RawActiveWindow {
    fn from(value: active_win_pos_rs::ActiveWindow) -> Self {
        RawActiveWindow {
            window_data: MaybeLocked::Normal(RawWindowData {
                identifiers: WindowIdentifiers {
                    app_name: value.app_name,
                    window_title: value.title,
                },
            }),
        }
    }
}

impl RepresentsWindow for RawActiveWindow {
    fn window_name(&self) -> WindowName {
        match &self.window_data {
            MaybeLocked::Locked => WindowName::Locked,
            MaybeLocked::Normal(data) => WindowName::Normal(data.identifiers.app_name.clone()),
        }
    }
}
