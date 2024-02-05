use crate::sensors::outputs::sensor_outputs::SensorOutputs;

use super::{
    expression::{EvalData, EvalResult, EvaluationError},
    Update,
};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone)]
#[serde(
    from = "persistence::VersionedValuePointer",
    into = "persistence::VersionedValuePointer"
)]
pub enum ValuePointer {
    OnlineStatus(TimeSpecifier),
    LockStatus(TimeSpecifier),
    ConstBool(bool),
    ConstUserId(UniqueUserId),
    UserId,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum TimeSpecifier {
    Last,
    Current,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum Value {
    Bool(bool),
    UserId(UniqueUserId),
}

impl ValuePointer {
    pub fn get_value(&self, data: &EvalData<'_, '_, '_>) -> Option<Value> {
        match self {
            ValuePointer::OnlineStatus(time_specifier) => {
                get_outputs_by_time_specifier(&data.update, time_specifier)
                    .get_online_status()
                    .map(|v| Value::Bool(v.online))
            }
            ValuePointer::LockStatus(time) => get_outputs_by_time_specifier(&data.update, time)
                .is_locked()
                .map(Value::Bool),
            ValuePointer::ConstBool(val) => Some(Value::Bool(*val)),
            ValuePointer::UserId => Some(Value::UserId(data.user.clone())),
            ValuePointer::ConstUserId(val) => Some(Value::UserId(val.clone())),
        }
    }
}

fn get_outputs_by_time_specifier<'a>(
    update: &'a Update<&SensorOutputs>,
    specifier: &TimeSpecifier,
) -> &'a SensorOutputs {
    match specifier {
        TimeSpecifier::Last => update.original,
        TimeSpecifier::Current => update.updated,
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
    use pro_serde_versioned::{Upgrade, VersionedUpgrade};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, VersionedUpgrade, PartialEq, Clone)]
    pub enum VersionedValuePointer {
        V1(ValuePointerV1),
        V2(ValuePointerV2),
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone)]
    pub enum ValuePointerV1 {
        LastOnlineStatus,
        CurrentOnlineStatus,
        ConstBool(bool),
        ConstUserId(UniqueUserId),
        UserId,
    }

    #[derive(Serialize, Deserialize, PartialEq, Clone)]
    pub enum ValuePointerV2 {
        OnlineStatus(TimeSpecifier),
        LockStatus(TimeSpecifier),
        ConstBool(bool),
        ConstUserId(UniqueUserId),
        UserId,
    }

    impl From<VersionedValuePointer> for ValuePointer {
        fn from(value: VersionedValuePointer) -> Self {
            let value = value.upgrade_to_latest();
            match value {
                ValuePointerV2::OnlineStatus(time) => Self::OnlineStatus(time),
                ValuePointerV2::LockStatus(time) => Self::LockStatus(time),
                ValuePointerV2::ConstBool(b) => Self::ConstBool(b),
                ValuePointerV2::ConstUserId(id) => Self::ConstUserId(id),
                ValuePointerV2::UserId => Self::UserId,
            }
        }
    }

    impl From<ValuePointer> for VersionedValuePointer {
        fn from(value: ValuePointer) -> Self {
            Self::V2(match value {
                ValuePointer::OnlineStatus(time) => ValuePointerV2::OnlineStatus(time),
                ValuePointer::LockStatus(time) => ValuePointerV2::LockStatus(time),
                ValuePointer::ConstBool(b) => ValuePointerV2::ConstBool(b),
                ValuePointer::ConstUserId(id) => ValuePointerV2::ConstUserId(id),
                ValuePointer::UserId => ValuePointerV2::UserId,
            })
        }
    }

    impl From<ValuePointerV1> for ValuePointer {
        fn from(value: ValuePointerV1) -> Self {
            VersionedValuePointer::V1(value).into()
        }
    }

    impl Upgrade<ValuePointerV2> for ValuePointerV1 {
        fn upgrade(self) -> ValuePointerV2 {
            match self {
                ValuePointerV1::LastOnlineStatus => {
                    ValuePointerV2::OnlineStatus(TimeSpecifier::Last)
                }
                ValuePointerV1::CurrentOnlineStatus => {
                    ValuePointerV2::OnlineStatus(TimeSpecifier::Current)
                }
                ValuePointerV1::ConstBool(b) => ValuePointerV2::ConstBool(b),
                ValuePointerV1::ConstUserId(id) => ValuePointerV2::ConstUserId(id),
                ValuePointerV1::UserId => ValuePointerV2::UserId,
            }
        }
    }
}
