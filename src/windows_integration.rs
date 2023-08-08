use windows::Win32::UI::WindowsAndMessaging::{
    MSG,
    WM_WTSSESSION_CHANGE, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};
use winit::platform::windows::EventLoopBuilderExtWindows;
use windows::Win32::System::RemoteDesktop::{WTSRegisterSessionNotification, NOTIFY_FOR_ALL_SESSIONS};
use windows::Win32::Foundation::HWND;

pub fn event_builder_hook(builder: &mut eframe::EventLoopBuilder<eframe::UserEvent>) -> () {
	builder.with_msg_hook(|msg| {
        let msg = msg as *const MSG;
        unsafe {
            // https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change
            if (*msg).message == WM_WTSSESSION_CHANGE
            {
                if (*msg).wParam.0 == WTS_SESSION_LOCK.try_into().unwrap() {
                    println!("Locked!");
                }

                if (*msg).wParam.0 == WTS_SESSION_UNLOCK.try_into().unwrap() {
                    println!("Unlocked!");
                }
            }
        }

        false
    });
}

pub fn setup_windows_integration(handle: raw_window_handle::Win32WindowHandle) {
    if !handle.hwnd.is_null() {
        println!("We have a hwnd?");

        let asdf: isize = handle.hwnd as isize;
        unsafe {
            WTSRegisterSessionNotification(HWND(asdf), NOTIFY_FOR_ALL_SESSIONS);
        }
    }
}