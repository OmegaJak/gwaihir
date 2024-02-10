use super::{notification_template::NotificationTemplate, TriggerContext};
use crate::notification::NotificationDispatch;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    ShowNotification(NotificationTemplate),
    //SetSummary(SummaryTemplate)
    // SetStatus(StatusTemplate),
}

impl Default for Action {
    fn default() -> Self {
        Action::ShowNotification(NotificationTemplate::new("".to_owned(), "".to_owned()))
    }
}

impl Action {
    pub fn execute<T: NotificationDispatch>(&self, context: &TriggerContext<'_, T>) {
        match self {
            Action::ShowNotification(template) => template.show_notification(context),
        }
    }
}
