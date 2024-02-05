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
pub use notification_template::NotificationTemplate;
pub use trigger::{BehaviorOnTrigger, Trigger, TriggerSource};
pub use trigger_manager::persistence::{TriggerManagerV1, VersionedTriggerManager};
pub use trigger_manager::TriggerManager;

#[derive(new, Clone)]
pub struct Update<T> {
    original: T,
    updated: T,
}

#[derive(Serialize)]
struct TriggerContext {
    user: String,
}
