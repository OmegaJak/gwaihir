use super::{expression::EvalData, Trigger, TriggerContext, Update};
use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct TriggerManager {
    triggers: HashMap<Uuid, Trigger>,
}

impl TriggerManager {
    pub fn remove_trigger(&mut self, predicate: impl Fn(&Trigger) -> bool) {
        self.triggers.retain(|_, c| !predicate(c));
    }

    pub fn remove_trigger_by_id(&mut self, trigger_id: &Uuid) -> Option<Trigger> {
        self.triggers.remove(trigger_id)
    }

    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.insert(Uuid::new_v4(), trigger);
    }

    pub fn get_serialized_triggers(&self) -> Vec<(Uuid, String)> {
        self.triggers
            .iter()
            .map(|(k, v)| (*k, ron::to_string(&v).unwrap()))
            .collect()
    }

    pub fn execute_triggers(
        &mut self,
        user_id: &UniqueUserId,
        user_display_name: String,
        update: Update<&SensorOutputs>,
    ) {
        let eval_data = EvalData {
            user: user_id,
            update,
        };
        let trigger_context = TriggerContext {
            user: user_display_name,
        };
        self.triggers
            .retain(|_, el| match el.criteria.evaluate(&eval_data) {
                Ok(val) => {
                    if val {
                        for action in el.actions.iter() {
                            action.execute(&trigger_context);
                        }
                    }

                    !el.drop_after_trigger || !val
                }
                Err(err) => {
                    log::error!("Failed to evaluate criteria: {}", err);
                    true
                }
            });
    }

    pub fn has_trigger(&self, predicate: impl Fn(&Trigger) -> bool) -> bool {
        self.triggers.iter().any(|(_, v)| predicate(v))
    }
}
