use super::expression::Expression;
use super::Action;
use derive_new::new;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(new, Serialize, Deserialize, Clone, PartialEq)]
#[serde(
    from = "persistence::VersionedTrigger",
    into = "persistence::VersionedTrigger"
)]
pub struct Trigger {
    pub name: String,
    pub enabled: bool,
    pub requested_users: HashMap<UniqueUserId, BehaviorOnTrigger>,
    pub source: TriggerSource,

    pub criteria: Expression,
    pub actions: Vec<Action>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum BehaviorOnTrigger {
    NoAction,
    Remove,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum TriggerSource {
    AppDefaults,
    User,
}

impl Trigger {
    pub fn requestable(&self) -> bool {
        matches!(
            &self.criteria,
            Expression::And(a, b) if matches!(**a, Expression::RequestedForUser) || matches!(**b, Expression::RequestedForUser)
        )
    }
}

pub mod persistence {
    use super::*;
    use crate::triggers::expression::persistence::ExpressionV1;
    use pro_serde_versioned::{Upgrade, VersionedUpgrade};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, VersionedUpgrade, Clone)]
    pub enum VersionedTrigger {
        V1(TriggerV1),
        V2(TriggerV2),
        V3(TriggerV3),
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TriggerV1 {
        pub criteria: ExpressionV1,
        pub drop_after_trigger: bool,
        pub actions: Vec<Action>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TriggerV2 {
        pub name: String,
        pub enabled: bool,
        pub requested_users: HashMap<UniqueUserId, BehaviorOnTrigger>,
        pub source: TriggerSource,

        pub criteria: ExpressionV1,
        pub actions: Vec<Action>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TriggerV3 {
        pub name: String,
        pub enabled: bool,
        pub requested_users: HashMap<UniqueUserId, BehaviorOnTrigger>,
        pub source: TriggerSource,

        pub criteria: Expression,
        pub actions: Vec<Action>,
    }

    impl From<VersionedTrigger> for Trigger {
        fn from(value: VersionedTrigger) -> Self {
            let value = value.upgrade_to_latest();
            Trigger {
                name: value.name,
                enabled: value.enabled,
                requested_users: value.requested_users,
                source: value.source,
                criteria: value.criteria,
                actions: value.actions,
            }
        }
    }

    impl From<Trigger> for VersionedTrigger {
        fn from(value: Trigger) -> Self {
            VersionedTrigger::V3(TriggerV3 {
                name: value.name,
                enabled: value.enabled,
                requested_users: value.requested_users,
                source: value.source,
                criteria: value.criteria,
                actions: value.actions,
            })
        }
    }

    impl Upgrade<TriggerV2> for TriggerV1 {
        fn upgrade(self) -> TriggerV2 {
            TriggerV2 {
                name: "{missing}".to_string(),
                enabled: true,
                requested_users: HashMap::new(),
                source: TriggerSource::User,
                criteria: self.criteria,
                actions: self.actions,
            }
        }
    }

    impl Upgrade<TriggerV3> for TriggerV2 {
        fn upgrade(self) -> TriggerV3 {
            TriggerV3 {
                name: self.name,
                enabled: self.enabled,
                requested_users: self.requested_users,
                source: self.source,
                criteria: self.criteria.into(),
                actions: self.actions,
            }
        }
    }
}
