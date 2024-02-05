use super::{trigger::BehaviorOnTrigger, value_pointer::ValuePointer, Update};
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
    RequestedForUser,
}

pub type ExpressionRef = std::rc::Rc<Expression>;

pub type EvalResult<T> = Result<T, EvaluationError>;

pub struct EvalData<'a, 'b, 'c> {
    pub user: &'a UniqueUserId,
    pub update: Update<&'b SensorOutputs>,
    pub requested_users: &'c HashMap<UniqueUserId, BehaviorOnTrigger>,
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Cannot compare a {0} to a {1}")]
    TypeMismatch(String, String),
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
                let left_value = left.get_value(data);
                let right_value = right.get_value(data);
                match (left_value, right_value) {
                    (Some(left), Some(right)) => left.equals(&right),
                    _ => EvalResult::Ok(false),
                }
            }
            Expression::RequestedForUser => {
                EvalResult::Ok(data.requested_users.contains_key(data.user))
            }
        }
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
        And(Rc<Expression>, Rc<Expression>),
        Or(Rc<Expression>, Rc<Expression>),
        Equals(ValuePointer, ValuePointer),
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
            match self {
                Self::And(a, b) => ExpressionV2::And(convert_rc(a), convert_rc(b)),
                Self::Or(a, b) => ExpressionV2::Or(convert_rc(a), convert_rc(b)),
                Self::Equals(a, b) => ExpressionV2::Equals(a.into(), b.into()),
                Self::RequestedForUser => ExpressionV2::RequestedForUser,
            }
        }
    }

    fn convert_rc<A, B>(a: Rc<A>) -> Rc<B>
    where
        B: From<A>,
    {
        Rc::new(Rc::into_inner(a).unwrap().into())
    }
}
