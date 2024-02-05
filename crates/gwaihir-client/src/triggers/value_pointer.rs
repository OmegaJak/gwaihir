use super::expression::{EvalData, EvalResult, EvaluationError};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(
    from = "persistence::VersionedValuePointer",
    into = "persistence::VersionedValuePointer"
)]
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

impl ValuePointer {
    pub fn get_value(&self, data: &EvalData<'_, '_, '_>) -> Option<Value> {
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
    pub fn equals(&self, other: &Value) -> EvalResult<bool> {
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

pub mod persistence {
    use super::*;
    use pro_serde_versioned::VersionedUpgrade;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, VersionedUpgrade, PartialEq, Clone)]
    pub enum VersionedValuePointer {
        V1(ValuePointerV1),
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone)]
    pub enum ValuePointerV1 {
        LastOnlineStatus,
        CurrentOnlineStatus,
        ConstBool(bool),
        ConstUserId(UniqueUserId),
        UserId,
    }

    impl From<VersionedValuePointer> for ValuePointer {
        fn from(value: VersionedValuePointer) -> Self {
            let value = value.upgrade_to_latest();
            match value {
                ValuePointerV1::LastOnlineStatus => Self::LastOnlineStatus,
                ValuePointerV1::CurrentOnlineStatus => Self::CurrentOnlineStatus,
                ValuePointerV1::ConstBool(b) => Self::ConstBool(b),
                ValuePointerV1::ConstUserId(id) => Self::ConstUserId(id),
                ValuePointerV1::UserId => Self::UserId,
            }
        }
    }

    impl From<ValuePointer> for VersionedValuePointer {
        fn from(value: ValuePointer) -> Self {
            Self::V1(match value {
                ValuePointer::LastOnlineStatus => ValuePointerV1::LastOnlineStatus,
                ValuePointer::CurrentOnlineStatus => ValuePointerV1::CurrentOnlineStatus,
                ValuePointer::ConstBool(b) => ValuePointerV1::ConstBool(b),
                ValuePointer::ConstUserId(id) => ValuePointerV1::ConstUserId(id),
                ValuePointer::UserId => ValuePointerV1::UserId,
            })
        }
    }

    impl From<ValuePointerV1> for ValuePointer {
        fn from(value: ValuePointerV1) -> Self {
            VersionedValuePointer::V1(value).into()
        }
    }
}
