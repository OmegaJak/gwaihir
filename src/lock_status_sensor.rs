use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, NOTIFY_FOR_ALL_SESSIONS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MSG, WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};
use winit::platform::windows::EventLoopBuilderExtWindows;

pub enum SessionEvent {
    Locked,
    Unlocked,
}

#[derive(Error, Debug)]
pub enum LockStatusSensorError {
    #[error("The provided window handle was null")]
    NullWindowHandle,
}

type EventLoopBuilder = eframe::EventLoopBuilder<eframe::UserEvent>;
type WindowHandle = raw_window_handle::Win32WindowHandle;
type EventBuffer = Arc<Mutex<VecDeque<SessionEvent>>>;

pub struct LockStatusSensorBuilder {}
pub struct OSRegisteredLockStatusSensorBuilder {}
pub struct EventLoopRegisteredLockStatusSensorBuilder {
    event_buffer: EventBuffer,
}
pub struct FullyRegisteredLockStatusSensorBuilder {
    event_buffer: EventBuffer,
}
pub struct LockStatusSensor {
    event_buffer: EventBuffer,
}

impl LockStatusSensorBuilder {
    pub fn new() -> LockStatusSensorBuilder {
        LockStatusSensorBuilder {}
    }

    pub fn set_event_loop_builder(
        self,
        native_options: &mut eframe::NativeOptions,
    ) -> Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>> {
        let registered_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>> =
            Rc::new(RefCell::new(None));
        let builder_clone = registered_builder.clone();

        native_options.event_loop_builder = Some(Box::new(move |builder| {
            builder_clone.replace(Some(self.register_msg_hook(builder)));
        }));

        registered_builder
    }

    pub fn register_msg_hook(
        self,
        builder: &mut EventLoopBuilder,
    ) -> EventLoopRegisteredLockStatusSensorBuilder {
        let event_buffer = create_event_buffer();
        register_msg_hook(builder, event_buffer.clone());
        EventLoopRegisteredLockStatusSensorBuilder { event_buffer }
    }

    pub fn register_os_hook(
        handle: WindowHandle,
    ) -> Result<OSRegisteredLockStatusSensorBuilder, LockStatusSensorError> {
        register_os_hook(handle)?;
        Ok(OSRegisteredLockStatusSensorBuilder {})
    }
}

impl EventLoopRegisteredLockStatusSensorBuilder {
    pub fn register_os_hook(
        self,
        handle: WindowHandle,
    ) -> Result<FullyRegisteredLockStatusSensorBuilder, LockStatusSensorError> {
        register_os_hook(handle)?;
        Ok(FullyRegisteredLockStatusSensorBuilder {
            event_buffer: self.event_buffer,
        })
    }
}

impl OSRegisteredLockStatusSensorBuilder {
    pub fn register_msg_hook(
        self,
        builder: &mut EventLoopBuilder,
    ) -> FullyRegisteredLockStatusSensorBuilder {
        let event_buffer = create_event_buffer();
        register_msg_hook(builder, event_buffer.clone());
        FullyRegisteredLockStatusSensorBuilder { event_buffer }
    }
}

impl FullyRegisteredLockStatusSensorBuilder {
    pub fn build(self) -> LockStatusSensor {
        LockStatusSensor {
            event_buffer: self.event_buffer,
        }
    }
}

impl LockStatusSensor {
    pub fn recv(&mut self) -> Option<SessionEvent> {
        self.event_buffer.lock().unwrap().pop_front()
    }
}

fn create_event_buffer() -> EventBuffer {
    EventBuffer::new(Mutex::new(VecDeque::new()))
}

fn register_os_hook(
    handle: raw_window_handle::Win32WindowHandle,
) -> Result<(), LockStatusSensorError> {
    if handle.hwnd.is_null() {
        return Err(LockStatusSensorError::NullWindowHandle);
    }

    let hwnd: isize = handle.hwnd as isize;
    unsafe {
        WTSRegisterSessionNotification(HWND(hwnd), NOTIFY_FOR_ALL_SESSIONS);
    }

    Ok(())
}

fn register_msg_hook(
    builder: &mut eframe::EventLoopBuilder<eframe::UserEvent>,
    event_buffer: EventBuffer,
) -> () {
    builder.with_msg_hook(move |msg| {
        let disable_winit_default_processing = false;
        if msg.is_null() {
            return disable_winit_default_processing;
        }

        let msg = msg as *const MSG;
        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change
            if (*msg).message == WM_WTSSESSION_CHANGE {
                if (*msg).wParam.0 == WTS_SESSION_LOCK.try_into().unwrap() {
                    event_buffer.lock().unwrap().push_back(SessionEvent::Locked)
                }

                if (*msg).wParam.0 == WTS_SESSION_UNLOCK.try_into().unwrap() {
                    event_buffer
                        .lock()
                        .unwrap()
                        .push_back(SessionEvent::Unlocked);
                }
            }
        }

        disable_winit_default_processing
    });
}
