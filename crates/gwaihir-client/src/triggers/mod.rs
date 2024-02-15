use derive_new::new;
use gwaihir_client_lib::UniqueUserId;

mod action;
mod expression;
mod notification_template;
mod summary_template;
mod text_template;
mod trigger;
mod trigger_manager;
pub mod ui;
mod value_pointer;

pub use action::Action;
pub use expression::Expression;
pub use expression::ExpressionRef;
pub use notification_template::NotificationTemplate;
pub use text_template::TextTemplateError;
pub use trigger::{BehaviorOnTrigger, Trigger, TriggerSource};
pub use trigger_manager::persistence::{TriggerManagerV1, VersionedTriggerManager};
pub use trigger_manager::TriggerManager;
pub use value_pointer::TimeSpecifier;
pub use value_pointer::ValuePointer;

use crate::user_summaries::UserSummaries;

#[derive(new, Clone)]
pub struct Update<T> {
    original: T,
    updated: T,
}

#[cfg(test)]
impl<T> Update<T> {
    pub(crate) fn as_ref(&self) -> Update<&T> {
        Update::new(&self.original, &self.updated)
    }
}

pub struct TriggerContext<'a, 'b, T> {
    user: String,
    user_id: UniqueUserId,
    notification_dispatch: &'a T,
    user_summaries: &'b mut UserSummaries,
}
