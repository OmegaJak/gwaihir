use super::{
    trigger::BehaviorOnTrigger,
    value_pointer::{Value, ValuePointer},
    Update,
};
use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(
    from = "persistence::VersionedExpression",
    into = "persistence::VersionedExpression"
)]
pub enum Expression {
    And(ExpressionRef, ExpressionRef),
    Or(ExpressionRef, ExpressionRef),
    Equals(ValuePointer, ValuePointer),
    NotEquals(ValuePointer, ValuePointer),
    GreaterThan(ValuePointer, ValuePointer),
    LessThan(ValuePointer, ValuePointer),
    GreaterThanOrEquals(ValuePointer, ValuePointer),
    LessThanOrEquals(ValuePointer, ValuePointer),
    RequestedForUser,
}

pub type ExpressionRef = Box<Expression>;

pub type EvalResult<T> = Result<T, EvaluationError>;

pub struct EvalData<'a, 'b, 'c> {
    pub user: &'a UniqueUserId,
    pub update: Update<&'b SensorOutputs>,
    pub requested_users: &'c HashMap<UniqueUserId, BehaviorOnTrigger>,
}

#[derive(Debug)]
pub enum OperationType {
    GreaterThan,
}

#[derive(Debug)]
pub enum ValueType {
    Bool,
    UserId,
    F64,
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Cannot compare a \"{0:#?}\" to a \"{1:#?}\"")]
    TypeMismatch(ValueType, ValueType),
    #[error("Invalid operation - \"{0:#?}\" is not a valid operation on a \"{1:#?}\"")]
    InvalidOperation(OperationType, ValueType),
}

impl Expression {
    pub fn evaluate(&self, data: &EvalData<'_, '_, '_>) -> EvalResult<bool> {
        match self {
            Expression::And(left, right) => {
                EvalResult::Ok(left.evaluate(data)? && right.evaluate(data)?)
            }
            Expression::Or(left, right) => {
                EvalResult::Ok(left.evaluate(data)? || right.evaluate(data)?)
            }
            Expression::Equals(left, right) => {
                binary_operator(data, left, right, |l, r| l.equals(r))
            }
            Expression::NotEquals(l, r) => binary_operator(data, l, r, |l, r| l.not_equals(r)),
            Expression::RequestedForUser => {
                EvalResult::Ok(data.requested_users.contains_key(data.user))
            }
            Expression::GreaterThan(left, right) => {
                binary_operator(data, left, right, |l, r| l.greater_than(r))
            }
            Expression::LessThan(left, right) => {
                binary_operator(data, left, right, |l, r| l.less_than(r))
            }
            Expression::GreaterThanOrEquals(left, right) => {
                binary_operator(data, left, right, |l, r| {
                    EvalResult::Ok(l.greater_than(r)? || l.equals(r)?)
                })
            }
            Expression::LessThanOrEquals(left, right) => {
                binary_operator(data, left, right, |l, r| {
                    EvalResult::Ok(l.less_than(r)? || l.equals(r)?)
                })
            }
        }
    }
}

fn binary_operator(
    data: &EvalData<'_, '_, '_>,
    left: &ValuePointer,
    right: &ValuePointer,
    evaluate: impl Fn(&Value, &Value) -> EvalResult<bool>,
) -> Result<bool, EvaluationError> {
    let left_value = left.get_value(data);
    let right_value = right.get_value(data);
    match (left_value, right_value) {
        (Some(left), Some(right)) => evaluate(&left, &right),
        _ => EvalResult::Ok(false),
    }
}

pub mod persistence {
    use crate::triggers::value_pointer::persistence::ValuePointerV1;

    use super::*;
    use pro_serde_versioned::{Upgrade, VersionedUpgrade};
    use serde::{Deserialize, Serialize};
    use std::rc::Rc;

    #[derive(Serialize, Deserialize, VersionedUpgrade, PartialEq, Clone)]
    pub enum VersionedExpression {
        V1(ExpressionV1),
        V2(ExpressionV2),
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone)]
    pub enum ExpressionV1 {
        And(Rc<ExpressionV1>, Rc<ExpressionV1>),
        Or(Rc<ExpressionV1>, Rc<ExpressionV1>),
        Equals(ValuePointerV1, ValuePointerV1),
        RequestedForUser,
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone)]
    pub enum ExpressionV2 {
        And(Box<Expression>, Box<Expression>),
        Or(Box<Expression>, Box<Expression>),
        Equals(ValuePointer, ValuePointer),
        NotEquals(ValuePointer, ValuePointer),
        GreaterThan(ValuePointer, ValuePointer),
        LessThan(ValuePointer, ValuePointer),
        GreaterThanOrEquals(ValuePointer, ValuePointer),
        LessThanOrEquals(ValuePointer, ValuePointer),
        RequestedForUser,
    }

    impl From<VersionedExpression> for Expression {
        fn from(value: VersionedExpression) -> Self {
            let value = value.upgrade_to_latest();
            match value {
                ExpressionV2::And(a, b) => Self::And(a, b),
                ExpressionV2::Or(a, b) => Self::Or(a, b),
                ExpressionV2::Equals(a, b) => Self::Equals(a, b),
                ExpressionV2::RequestedForUser => Self::RequestedForUser,
                ExpressionV2::NotEquals(a, b) => Self::NotEquals(a, b),
                ExpressionV2::GreaterThan(a, b) => Self::GreaterThan(a, b),
                ExpressionV2::LessThan(a, b) => Self::LessThan(a, b),
                ExpressionV2::GreaterThanOrEquals(a, b) => Self::GreaterThanOrEquals(a, b),
                ExpressionV2::LessThanOrEquals(a, b) => Self::LessThanOrEquals(a, b),
            }
        }
    }

    impl From<Expression> for VersionedExpression {
        fn from(value: Expression) -> Self {
            Self::V2(match value {
                Expression::And(a, b) => ExpressionV2::And(a, b),
                Expression::Or(a, b) => ExpressionV2::Or(a, b),
                Expression::Equals(a, b) => ExpressionV2::Equals(a, b),
                Expression::RequestedForUser => ExpressionV2::RequestedForUser,
                Expression::NotEquals(a, b) => ExpressionV2::NotEquals(a, b),
                Expression::GreaterThan(a, b) => ExpressionV2::GreaterThan(a, b),
                Expression::LessThan(a, b) => ExpressionV2::LessThan(a, b),
                Expression::GreaterThanOrEquals(a, b) => ExpressionV2::GreaterThanOrEquals(a, b),
                Expression::LessThanOrEquals(a, b) => ExpressionV2::LessThanOrEquals(a, b),
            })
        }
    }

    impl From<ExpressionV1> for Expression {
        fn from(value: ExpressionV1) -> Self {
            VersionedExpression::V1(value).into()
        }
    }

    impl Upgrade<ExpressionV2> for ExpressionV1 {
        fn upgrade(self) -> ExpressionV2 {
            fn rc_to_box<A, B>(a: Rc<A>) -> Box<B>
            where
                B: From<A>,
            {
                Box::new(Rc::into_inner(a).unwrap().into())
            }

            match self {
                Self::And(a, b) => ExpressionV2::And(rc_to_box(a), rc_to_box(b)),
                Self::Or(a, b) => ExpressionV2::Or(rc_to_box(a), rc_to_box(b)),
                Self::Equals(a, b) => ExpressionV2::Equals(a.into(), b.into()),
                Self::RequestedForUser => ExpressionV2::RequestedForUser,
            }
        }
    }
}
