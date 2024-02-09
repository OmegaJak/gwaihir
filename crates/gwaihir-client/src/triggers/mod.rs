use derive_new::new;
use serde::Serialize;

mod action;
mod expression;
mod notification_template;
mod trigger;
mod trigger_manager;
mod value_pointer;

pub use action::Action;
pub use expression::Expression;
pub use expression::ExpressionRef;
pub use notification_template::NotificationTemplate;
pub use notification_template::NotificationTemplateError;
pub use trigger::{BehaviorOnTrigger, Trigger, TriggerSource};
pub use trigger_manager::persistence::{TriggerManagerV1, VersionedTriggerManager};
pub use trigger_manager::TriggerManager;
pub use value_pointer::TimeSpecifier;
pub use value_pointer::ValuePointer;

use crate::notification::NotificationDispatch;

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

#[derive(Serialize)]
pub struct TriggerContext<'a, T: NotificationDispatch> {
    user: String,
    notification_dispatch: &'a T,
}
