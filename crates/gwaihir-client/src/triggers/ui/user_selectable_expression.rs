use crate::triggers::{Expression, TimeSpecifier, ValuePointer};
use enum_iterator::Sequence;
use gwaihir_client_lib::UniqueUserId;

#[derive(Clone, PartialEq, Sequence)]
pub enum UserSelectableExpression {
    OnlineStatus,
    LockStatus,
    TotalKeyboardMouseUsage,
    UserId,
}

impl UserSelectableExpression {
    pub fn get_default(&self) -> Expression {
        match self {
            UserSelectableExpression::OnlineStatus => Expression::Equals(
                ValuePointer::OnlineStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(true),
            ),
            UserSelectableExpression::LockStatus => Expression::Equals(
                ValuePointer::LockStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(true),
            ),
            UserSelectableExpression::TotalKeyboardMouseUsage => Expression::Equals(
                ValuePointer::TotalKeyboardMouseUsage(TimeSpecifier::Current),
                ValuePointer::ConstF64(0.0),
            ),
            UserSelectableExpression::UserId => Expression::Equals(
                ValuePointer::UserId,
                ValuePointer::ConstUserId(UniqueUserId::new("")),
            ),
        }
    }
}

impl std::fmt::Display for UserSelectableExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserSelectableExpression::OnlineStatus => write!(f, "Online Status"),
            UserSelectableExpression::LockStatus => write!(f, "Lock Status"),
            UserSelectableExpression::TotalKeyboardMouseUsage => write!(f, "Total KB/M Usage"),
            UserSelectableExpression::UserId => write!(f, "User Id"),
        }
    }
}
