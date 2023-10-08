use super::{LockStatusSensorError, SessionEvent};
use std::sync::mpsc::Sender;

pub fn register_os_hook(_handle: ()) -> Result<(), LockStatusSensorError> {
    Ok(())
}

pub fn register_msg_hook(
    _builder: &mut eframe::EventLoopBuilder<eframe::UserEvent>,
    _tx_to_sensor: Sender<SessionEvent>,
) {
}
