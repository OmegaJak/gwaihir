use super::{
    notification_template::NotificationTemplate, summary_template::SummaryTemplate, TriggerContext,
};
use crate::notification::NotificationDispatch;
use log_err::LogErrResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum Action {
    ShowNotification(NotificationTemplate),
    SetSummary(SummaryTemplate),
}

impl Default for Action {
    fn default() -> Self {
        Action::ShowNotification(Default::default())
    }
}

impl Action {
    pub fn execute<T: NotificationDispatch>(&self, context: &mut TriggerContext<'_, '_, T>) {
        match self {
            Action::ShowNotification(template) => template.show_notification(context).log_unwrap(),
            Action::SetSummary(summary) => {
                let rendered = summary.summary.render(context).log_unwrap();
                context
                    .user_summaries
                    .set_summary(context.user_id.clone(), rendered);
            }
        }
    }
}
