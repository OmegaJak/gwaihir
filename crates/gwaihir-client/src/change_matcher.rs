use std::cell::RefCell;

use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use derive_new::new;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Default, Serialize, Deserialize)]
pub struct ChangeMatcher {
    matchers: Vec<Matcher>,
}

#[derive(new, Serialize, Deserialize)]
pub struct Matcher {
    pub criteria: Expression,
    pub drop_after_match: bool,
}

#[derive(new)]
pub struct Update<T> {
    original: T,
    updated: T,
}

impl ChangeMatcher {
    pub fn match_once_when_online(&mut self, user_id: UniqueUserId) {
        self.add_match_once(user_comes_online_expression(user_id));
    }

    pub fn remove_matcher(&mut self, predicate: impl Fn(&Matcher) -> bool) {
        self.matchers.retain(|c| !predicate(c));
    }

    pub fn add_match_once(&mut self, criteria: Expression) {
        self.matchers.push(Matcher::new(criteria, true));
    }

    pub fn add_match(&mut self, criteria: Expression) {
        self.matchers.push(Matcher::new(criteria, false));
    }

    pub fn get_matches(
        &mut self,
        user_id: &UniqueUserId,
        update: Update<&SensorOutputs>,
    ) -> Vec<UniqueUserId> {
        //TODO: Couldn't decide what this should return, so I'm being lazy. Figure out what it should actually do
        let mut matched: Vec<UniqueUserId> = Vec::new();

        let eval_data = EvalData {
            user: user_id,
            update,
        };
        self.matchers
            .retain(|el| match el.criteria.evaluate(&eval_data) {
                Ok(val) => {
                    if val {
                        matched.push(user_id.clone())
                    }

                    !el.drop_after_match || !val
                }
                Err(err) => {
                    log::error!("Failed to evaluate criteria: {}", err);
                    true
                }
            });

        matched
    }

    pub fn has_matcher(&self, predicate: impl Fn(&Matcher) -> bool) -> bool {
        self.matchers.iter().any(predicate)
    }
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

struct EvalData<'a, 'b> {
    user: &'a UniqueUserId,
    update: Update<&'b SensorOutputs>,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Expression {
    And(ExpressionRef, ExpressionRef),
    Or(ExpressionRef, ExpressionRef),
    Equals(ValuePointer, ValuePointer),
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum ValuePointer {
    LastOnlineStatus,
    CurrentOnlineStatus,
    ConstBool(bool),
    ConstUserId(UniqueUserId),
    UserId,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Value {
    Bool(bool),
    UserId(UniqueUserId),
}

pub type ExpressionRef = std::rc::Rc<std::cell::RefCell<Expression>>;

type EvalResult<T> = Result<T, EvaluationError>;

fn new_expr_ref(expr: Expression) -> ExpressionRef {
    ExpressionRef::new(RefCell::new(expr))
}

#[derive(Error, Debug)]
pub enum EvaluationError {
    #[error("Cannot compare a {0} to a {1}")]
    TypeMismatch(String, String),
}

impl Expression {
    fn evaluate(&self, data: &EvalData<'_, '_>) -> EvalResult<bool> {
        match self {
            Expression::And(left, right) => {
                EvalResult::Ok(left.borrow().evaluate(data)? && right.borrow().evaluate(data)?)
            }
            Expression::Or(left, right) => {
                EvalResult::Ok(left.borrow().evaluate(data)? || right.borrow().evaluate(data)?)
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
