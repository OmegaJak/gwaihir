use self::expression::{Expression, ExpressionRef, ValuePointer};
use derive_new::new;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

mod expression;
mod trigger_manager;

pub use trigger_manager::TriggerManager;

#[derive(new, Serialize, Deserialize, Clone)]
pub struct Trigger {
    pub criteria: Expression,
    pub drop_after_trigger: bool,
}

#[derive(new)]
pub struct Update<T> {
    original: T,
    updated: T,
}

pub fn user_comes_online_expression(user_id: UniqueUserId) -> Expression {
    Expression::And(
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
    )
}

fn new_expr_ref(expr: Expression) -> ExpressionRef {
    ExpressionRef::new(expr)
}
