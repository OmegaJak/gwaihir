use super::{notification_template::NotificationTemplate, TriggerContext};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    ShowNotification(NotificationTemplate),
    //SetSummary(SummaryTemplate)
    // SetStatus(StatusTemplate),
}

impl Action {
    pub(super) fn execute(&self, context: &TriggerContext) {
        match self {
            Action::ShowNotification(template) => template.show_notification(context),
        }
    }
}
