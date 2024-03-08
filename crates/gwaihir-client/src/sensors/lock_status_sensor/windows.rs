use super::{LockStatusSensorError, SessionEvent};
use std::sync::mpsc::Sender;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::RemoteDesktop::{
    WTSRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    MSG, WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};
use winit::platform::windows::EventLoopBuilderExtWindows;

pub fn register_os_hook(
    handle: raw_window_handle::Win32WindowHandle,
) -> Result<(), LockStatusSensorError> {
    let hwnd: isize = isize::from(handle.hwnd);
    unsafe {
        WTSRegisterSessionNotification(HWND(hwnd), NOTIFY_FOR_THIS_SESSION);
    }

    Ok(())
}

pub fn register_msg_hook(
    builder: &mut eframe::EventLoopBuilder<eframe::UserEvent>,
    tx_to_sensor: Sender<SessionEvent>,
) {
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
