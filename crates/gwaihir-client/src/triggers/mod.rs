use self::expression::{Expression, ExpressionRef, ValuePointer};
use derive_new::new;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

mod action;
mod expression;
mod notification_template;
mod trigger_manager;

pub use action::Action;
pub use notification_template::NotificationTemplate;
pub use trigger_manager::TriggerManager;

#[derive(new, Serialize, Deserialize, Clone, PartialEq)]
pub struct Trigger {
    pub criteria: Expression,
    pub drop_after_trigger: bool,
    pub actions: Vec<Action>,
}

#[derive(new)]
pub struct Update<T> {
    original: T,
    updated: T,
}

#[derive(Serialize)]
struct TriggerContext {
    user: String,
}

pub fn user_comes_online_trigger(user_id: UniqueUserId, drop_after_trigger: bool) -> Trigger {
    let criteria = Expression::And(
        new_expr_ref(Expression::And(
            new_expr_ref(Expression::Equals(
                ValuePointer::LastOnlineStatus,
                ValuePointer::ConstBool(false),
            )),
            new_expr_ref(Expression::Equals(
                ValuePointer::CurrentOnlineStatus,
                ValuePointer::ConstBool(true),
            )),
        )),
        new_expr_ref(Expression::Equals(
            ValuePointer::UserId,
            ValuePointer::ConstUserId(user_id),
        )),
    );
    let actions = vec![Action::ShowNotification(NotificationTemplate::new(
        "{{user}} now Online".to_string(),
        "The user \"{{user}}\" has transitioned from offline to online".to_string(),
    ))];
    Trigger {
        criteria,
        drop_after_trigger,
        actions,
    }
}

fn new_expr_ref(expr: Expression) -> ExpressionRef {
    ExpressionRef::new(expr)
}
