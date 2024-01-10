use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::sensors::outputs::sensor_outputs::SensorOutputs;

use super::Update;

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Expression {
    And(ExpressionRef, ExpressionRef),
    Or(ExpressionRef, ExpressionRef),
    Equals(ValuePointer, ValuePointer),
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum ValuePointer {
    LastOnlineStatus,
    CurrentOnlineStatus,
    ConstBool(bool),
    ConstUserId(UniqueUserId),
    UserId,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    UserId(UniqueUserId),
}

pub type ExpressionRef = std::rc::Rc<Expression>;

type EvalResult<T> = Result<T, EvaluationError>;

pub struct EvalData<'a, 'b> {
    pub user: &'a UniqueUserId,
    pub update: Update<&'b SensorOutputs>,
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Cannot compare a {0} to a {1}")]
    TypeMismatch(String, String),
}

impl Expression {
    pub fn evaluate(&self, data: &EvalData<'_, '_>) -> EvalResult<bool> {
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
        }
    }
}

impl ValuePointer {
    fn get_value(&self, data: &EvalData<'_, '_>) -> Option<Value> {
        match self {
            ValuePointer::LastOnlineStatus => data
                .update
                .original
                .get_online_status()
                .map(|v| Value::Bool(v.online)),
            ValuePointer::CurrentOnlineStatus => data
                .update
                .updated
                .get_online_status()
                .map(|v| Value::Bool(v.online)),
            ValuePointer::ConstBool(val) => Some(Value::Bool(*val)),
            ValuePointer::UserId => Some(Value::UserId(data.user.clone())),
            ValuePointer::ConstUserId(val) => Some(Value::UserId(val.clone())),
        }
    }
}

impl Value {
    fn equals(&self, other: &Value) -> EvalResult<bool> {
        match (self, other) {
            (Value::Bool(left), Value::Bool(right)) => EvalResult::Ok(left == right),
            (Value::Bool(_), Value::UserId(_)) => EvalResult::Err(EvaluationError::TypeMismatch(
                "bool".to_string(),
                "user id".to_string(),
            )),
            (Value::UserId(_), Value::Bool(_)) => EvalResult::Err(EvaluationError::TypeMismatch(
                "bool".to_string(),
                "user id".to_string(),
            )),
            (Value::UserId(left), Value::UserId(right)) => EvalResult::Ok(left == right),
        }
    }
}
