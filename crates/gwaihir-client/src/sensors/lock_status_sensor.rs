use super::Sensor;
use crate::sensor_outputs::lock_status::LockStatus;
use crate::sensor_outputs::SensorOutput;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
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

pub struct LockStatusSensorBuilder {}
// pub struct OSRegisteredLockStatusSensorBuilder {}
pub struct EventLoopRegisteredLockStatusSensorBuilder {
    sensor_rx: Receiver<SessionEvent>,
}
pub struct FullyRegisteredLockStatusSensorBuilder {
    sensor_rx: Receiver<SessionEvent>,
}

pub struct LockStatusSensor {
    session_event_rx: Receiver<SessionEvent>,
    lock_status: LockStatus,
}

impl LockStatusSensorBuilder {
    pub fn new() -> LockStatusSensorBuilder {
        LockStatusSensorBuilder {}
    }

    pub fn set_event_loop_builder(
        self,
        native_options: &mut eframe::NativeOptions,
    ) -> Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>> {
        let registered_builder = Rc::new(RefCell::new(None));
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
        let (tx_to_sensor, rx_from_windows) = mpsc::channel();
        register_msg_hook(builder, tx_to_sensor);
        EventLoopRegisteredLockStatusSensorBuilder {
            sensor_rx: rx_from_windows,
        }
    }
}

impl EventLoopRegisteredLockStatusSensorBuilder {
    pub fn register_os_hook(
        self,
        handle: WindowHandle,
    ) -> Result<FullyRegisteredLockStatusSensorBuilder, LockStatusSensorError> {
        register_os_hook(handle)?;
        Ok(FullyRegisteredLockStatusSensorBuilder {
            sensor_rx: self.sensor_rx,
        })
    }
}

impl FullyRegisteredLockStatusSensorBuilder {
    pub fn build(self) -> LockStatusSensor {
        LockStatusSensor {
            session_event_rx: self.sensor_rx,
            lock_status: Default::default(),
        }
    }
}

impl Sensor for LockStatusSensor {
    fn get_output(&mut self) -> SensorOutput {
        match self.session_event_rx.try_recv() {
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => todo!(),
            Ok(SessionEvent::Locked) => self.lock_status.num_locks += 1,
            Ok(SessionEvent::Unlocked) => self.lock_status.num_unlocks += 1,
        }

        SensorOutput::LockStatus(self.lock_status.clone())
    }
}

#[cfg(test)]
impl LockStatusSensor {
    pub fn new() -> (Self, Sender<SessionEvent>) {
        let (tx, rx) = mpsc::channel();
        (
            LockStatusSensor {
                session_event_rx: rx,
                lock_status: Default::default(),
            },
            tx,
        )
    }
}

fn register_os_hook(
    handle: raw_window_handle::Win32WindowHandle,
) -> Result<(), LockStatusSensorError> {
    if handle.hwnd.is_null() {
        return Err(LockStatusSensorError::NullWindowHandle);
    }

    let hwnd: isize = handle.hwnd as isize;
    unsafe {
        WTSRegisterSessionNotification(HWND(hwnd), NOTIFY_FOR_THIS_SESSION);
    }

    Ok(())
}

fn register_msg_hook(
    builder: &mut eframe::EventLoopBuilder<eframe::UserEvent>,
    tx_to_sensor: Sender<SessionEvent>,
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
                if (*msg).wParam.0 == TryInto::<usize>::try_into(WTS_SESSION_LOCK).unwrap() {
                    tx_to_sensor.send(SessionEvent::Locked).unwrap();
                }

                if (*msg).wParam.0 == TryInto::<usize>::try_into(WTS_SESSION_UNLOCK).unwrap() {
                    tx_to_sensor.send(SessionEvent::Unlocked).unwrap();
                }
            }
        }

        disable_winit_default_processing
    });
}
