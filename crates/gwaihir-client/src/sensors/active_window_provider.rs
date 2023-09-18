use std::{cell::RefCell, rc::Rc};

use super::outputs::window_activity::{RepresentsWindow, WindowName};

pub struct RawActiveWindow {
    pub window_name: WindowName,
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
                window_name: WindowName::Locked,
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
            window_name: WindowName::Normal(value.app_name),
        }
    }
}

impl RepresentsWindow for RawActiveWindow {
    fn window_name(&self) -> &WindowName {
        &self.window_name
    }
}
