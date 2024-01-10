use crate::sensors::outputs::sensor_outputs::SensorOutputs;

use super::{
    expression::{EvalData, Expression},
    user_comes_online_expression, Trigger, Update,
};
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize)]
pub struct TriggerManager {
    triggers: HashMap<Uuid, Trigger>,
}

impl TriggerManager {
    pub fn match_once_when_online(&mut self, user_id: UniqueUserId) {
        self.add_trigger_once(user_comes_online_expression(user_id));
    }

    pub fn remove_trigger(&mut self, predicate: impl Fn(&Trigger) -> bool) {
        self.triggers.retain(|_, c| !predicate(c));
    }

    pub fn remove_trigger_by_id(&mut self, matcher_id: &Uuid) -> Option<Trigger> {
        self.triggers.remove(matcher_id)
    }

    pub fn add_trigger_once(&mut self, criteria: Expression) {
        self.add_trigger(Trigger::new(criteria, true));
    }

    pub fn add_trigger_with_criteria(&mut self, criteria: Expression) {
        self.add_trigger(Trigger::new(criteria, false));
    }

    pub fn add_trigger(&mut self, matcher: Trigger) {
        self.triggers.insert(Uuid::new_v4(), matcher);
    }

    pub fn get_serialized_triggers(&self) -> Vec<(Uuid, String)> {
        self.triggers
            .iter()
            .map(|(k, v)| (*k, ron::to_string(&v.criteria).unwrap()))
            .collect()
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
        self.triggers
            .retain(|_, el| match el.criteria.evaluate(&eval_data) {
                Ok(val) => {
                    if val {
                        matched.push(user_id.clone())
                    }

                    !el.drop_after_trigger || !val
                }
                Err(err) => {
                    log::error!("Failed to evaluate criteria: {}", err);
                    true
                }
            });

        matched
    }

    pub fn has_trigger(&self, predicate: impl Fn(&Trigger) -> bool) -> bool {
        self.triggers.iter().any(|(_, v)| predicate(v))
    }
}
