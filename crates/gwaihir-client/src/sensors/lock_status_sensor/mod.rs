use crate::sensors::outputs::lock_status::LockStatus;
use crate::sensors::outputs::sensor_output::SensorOutput;
use crate::sensors::Sensor;
use log::error;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use thiserror::Error;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as sys;
#[cfg(target_os = "windows")]
type WindowHandle = raw_window_handle::Win32WindowHandle;
#[cfg(target_os = "windows")]
use winit::raw_window_handle::HasWindowHandle;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as sys;
#[cfg(target_os = "linux")]
type WindowHandle = ();

#[allow(unused)]
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

pub struct LockStatusSensorBuilder {}
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
        sys::register_msg_hook(builder, tx_to_sensor);
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
        sys::register_os_hook(handle)?;
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

#[allow(unreachable_code, unused_variables)]
pub fn init_lock_status_sensor(
    cc: &eframe::CreationContext<'_>,
    sensor_builder: Rc<RefCell<Option<EventLoopRegisteredLockStatusSensorBuilder>>>,
) -> Option<LockStatusSensor> {
    #[cfg(target_os = "windows")]
    return match cc.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(raw_handle) => match sensor_builder
                .take()
                .expect(
                    "The lock status sensor builder should be ready when\
                 we initialize the Template App",
                )
                .register_os_hook(raw_handle)
            {
                Ok(builder) => Some(builder.build()),
                Err(err) => {
                    error!("{:#?}", err);
                    None
                }
            },
            _ => panic!("Running on an unsupported version of Windows"),
        },
        Err(e) => {
            error!("{:#?}", e);
            None
        }
    };

    None
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
    pub fn new() -> (Self, mpsc::Sender<SessionEvent>) {
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
