use log_err::LogErrResult;

#[cfg_attr(test, mockall::automock)]
pub trait NotificationDispatch {
    fn show_notification(&self, summary: &str, body: &str);
}

pub struct OSNotificationDispatch;

impl NotificationDispatch for OSNotificationDispatch {
    fn show_notification(&self, summary: &str, body: &str) {
        notify_rust::Notification::new()
            .summary(summary)
            .body(body)
            .sound_name("Default")
            .show()
            .log_unwrap();
    }
}
