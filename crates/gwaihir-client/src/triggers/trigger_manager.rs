use super::{
    expression::EvalData,
    trigger::{BehaviorOnTrigger, TriggerSource},
    Trigger, TriggerContext, Update,
};
use crate::{notification::NotificationDispatch, sensors::outputs::sensor_outputs::SensorOutputs};
use gwaihir_client_lib::UniqueUserId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Default, Serialize, Deserialize, Clone)]
#[serde(
    from = "persistence::VersionedTriggerManager",
    into = "persistence::VersionedTriggerManager"
)]
pub struct TriggerManager {
    triggers: IndexMap<Uuid, Trigger>,
}

impl TriggerManager {
    pub fn remove_trigger_by_id(&mut self, trigger_id: &Uuid) -> Option<Trigger> {
        self.triggers.shift_remove(trigger_id)
    }

    pub fn get_trigger_by_id(&mut self, trigger_id: &Uuid) -> Option<&Trigger> {
        self.triggers.get(trigger_id)
    }

    pub fn add_trigger(&mut self, trigger: Trigger) {
        self.triggers.insert(Uuid::new_v4(), trigger);
    }

    pub fn move_trigger(&mut self, trigger_id: &Uuid, index_offset: isize) {
        if let Some(index) = self.triggers.get_index_of(trigger_id) {
            if let Some(other_index) = index.checked_add_signed(index_offset) {
                // Implicitly > 0
                if other_index < self.triggers.len() {
                    self.triggers.swap_indices(index, other_index);
                }
            }
        }
    }

    pub fn triggers_iter_mut(&mut self) -> impl Iterator<Item = (&Uuid, &mut Trigger)> {
        self.triggers.iter_mut()
    }

    pub fn execute_triggers(
        &mut self,
        user_id: &UniqueUserId,
        user_display_name: String,
        update: Update<&SensorOutputs>,
        notification_dispatch: &impl NotificationDispatch,
    ) {
        let trigger_context = TriggerContext {
            user: user_display_name,
            notification_dispatch,
        };
        for trigger in self.triggers.values_mut().filter(|t| t.enabled) {
            let eval_data = EvalData {
                user: user_id,
                update: update.clone(),
            };

            if !trigger.requestable || trigger.requested_users.contains_key(user_id) {
                match trigger.criteria.evaluate(&eval_data) {
                    Ok(true) => {
                        for action in trigger.actions.iter() {
                            action.execute(&trigger_context);
                        }

                        if let Some(behavior_on_trigger) = trigger.requested_users.get(user_id) {
                            match behavior_on_trigger {
                                BehaviorOnTrigger::NoAction => {}
                                BehaviorOnTrigger::Remove => {
                                    if trigger.requestable {
                                        trigger.requested_users.remove(user_id);
                                    }
                                }
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(err) => {
                        log::error!("Failed to evaluate criteria: {}", err);
                    }
                }
            }
        }
    }

    pub fn reset_default_triggers(&mut self) {
        self.triggers
            .retain(|_, trigger| trigger.source != TriggerSource::AppDefaults);
        self.add_trigger(default_triggers::user_coming_online());
        self.add_trigger(default_triggers::user_unlocked());
        self.add_trigger(default_triggers::user_active_again());
        self.add_trigger(default_triggers::done_with_meeting());
    }
}

pub mod persistence {
    use super::*;
    use crate::triggers::trigger::persistence::{TriggerV1, VersionedTrigger};
    use pro_serde_versioned::{Upgrade, VersionedUpgrade};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, VersionedUpgrade, Clone)]
    pub enum VersionedTriggerManager {
        V1(TriggerManagerV1),
        V2(TriggerManagerV2),
        V3(TriggerManagerV3),
    }

    #[derive(Serialize, Deserialize, Clone, Default)]
    pub struct TriggerManagerV1 {
        triggers: HashMap<Uuid, TriggerV1>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TriggerManagerV2 {
        triggers: HashMap<Uuid, Trigger>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct TriggerManagerV3 {
        triggers: IndexMap<Uuid, Trigger>,
    }

    impl From<VersionedTriggerManager> for TriggerManager {
        fn from(value: VersionedTriggerManager) -> Self {
            let value = value.upgrade_to_latest();
            TriggerManager {
                triggers: value.triggers,
            }
        }
    }

    impl From<TriggerManager> for VersionedTriggerManager {
        fn from(value: TriggerManager) -> Self {
            VersionedTriggerManager::V3(TriggerManagerV3 {
                triggers: value.triggers,
            })
        }
    }

    impl Upgrade<TriggerManagerV2> for TriggerManagerV1 {
        fn upgrade(self) -> TriggerManagerV2 {
            let mut new_map = HashMap::new();
            for (id, trigger_v1) in self.triggers {
                new_map.insert(id, VersionedTrigger::V1(trigger_v1).into());
            }

            TriggerManagerV2 { triggers: new_map }
        }
    }

    impl Upgrade<TriggerManagerV3> for TriggerManagerV2 {
        fn upgrade(self) -> TriggerManagerV3 {
            TriggerManagerV3 {
                triggers: IndexMap::from_iter(self.triggers),
            }
        }
    }
}

mod default_triggers {
    use crate::triggers::{
        trigger::TriggerSource,
        value_pointer::{TimeSpecifier, ValuePointer},
        Action, Expression, NotificationTemplate, Trigger,
    };

    pub fn user_coming_online() -> Trigger {
        let criteria = Expression::And(
            Expression::Equals(
                ValuePointer::OnlineStatus(TimeSpecifier::Last),
                ValuePointer::ConstBool(false),
            )
            .into(),
            Expression::Equals(
                ValuePointer::OnlineStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(true),
            )
            .into(),
        );
        let actions = vec![Action::ShowNotification(NotificationTemplate::new(
            "{{user}} now Online".to_string(),
            "The user \"{{user}}\" has transitioned from offline to online".to_string(),
        ))];
        let name = "User coming online".to_string();
        Trigger {
            criteria,
            enabled: true,
            requestable: true,
            requested_users: Default::default(),
            source: TriggerSource::AppDefaults,
            actions,
            name,
        }
    }

    pub fn user_unlocked() -> Trigger {
        let criteria = Expression::And(
            Expression::Equals(
                ValuePointer::LockStatus(TimeSpecifier::Last),
                ValuePointer::ConstBool(true),
            )
            .into(),
            Expression::Equals(
                ValuePointer::LockStatus(TimeSpecifier::Current),
                ValuePointer::ConstBool(false),
            )
            .into(),
        );
        let actions = vec![Action::ShowNotification(NotificationTemplate::new(
            "{{user}} unlocked".to_string(),
            "The user \"{{user}}\" has unlocked their computer".to_string(),
        ))];
        let name = "User unlocked".to_string();
        Trigger {
            criteria,
            enabled: true,
            requestable: true,
            requested_users: Default::default(),
            source: TriggerSource::AppDefaults,
            actions,
            name,
        }
    }

    pub fn user_active_again() -> Trigger {
        let criteria = Expression::And(
            Expression::Equals(
                ValuePointer::TotalKeyboardMouseUsage(TimeSpecifier::Last),
                ValuePointer::ConstF64(0.0),
            )
            .into(),
            Expression::GreaterThan(
                ValuePointer::TotalKeyboardMouseUsage(TimeSpecifier::Current),
                ValuePointer::ConstF64(0.0),
            )
            .into(),
        );
        let actions = vec![Action::ShowNotification(NotificationTemplate::new(
            "{{user}} is now active".to_string(),
            "\"{{user}}\" has used their mouse/keyboard for the first time in 10 minutes"
                .to_string(),
        ))];
        let name = "User active again".to_string();
        Trigger {
            criteria,
            enabled: true,
            requestable: true,
            requested_users: Default::default(),
            source: TriggerSource::AppDefaults,
            actions,
            name,
        }
    }

    pub fn done_with_meeting() -> Trigger {
        let criteria = Expression::And(
            Expression::GreaterThan(
                ValuePointer::NumAppsUsingMicrophone(TimeSpecifier::Last),
                ValuePointer::ConstUsize(0),
            )
            .into(),
            Expression::Equals(
                ValuePointer::NumAppsUsingMicrophone(TimeSpecifier::Current),
                ValuePointer::ConstUsize(0),
            )
            .into(),
        );
        let actions = vec![Action::ShowNotification(NotificationTemplate::new(
            "{{user}} is done with their meeting".to_string(),
            "The # of apps listening to \"{{user}}'s\" microphone has dropped to 0".to_string(),
        ))];
        let name = "Done with meeting".to_string();
        Trigger {
            criteria,
            enabled: true,
            requestable: true,
            requested_users: Default::default(),
            source: TriggerSource::AppDefaults,
            actions,
            name,
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::{
        notification::MockNotificationDispatch,
        triggers::{Action, Expression, NotificationTemplate, ValuePointer},
    };
    use lazy_static::lazy_static;
    use maplit::hashmap;

    lazy_static! {
        static ref REQUESTED_USER_ID: UniqueUserId = UniqueUserId::new("requested");
        static ref NOT_REQUESTED_USER_ID: UniqueUserId = UniqueUserId::new("not_requested");
    }

    #[test]
    pub fn execute_triggers_when_trigger_not_requestable_and_not_requested_executes_trigger() {
        let mut notification_dispatch = MockNotificationDispatch::new();
        let mut manager = TriggerManager::default();
        manager.add_trigger(Trigger {
            requestable: false,
            requested_users: hashmap!(),
            ..default_test_trigger()
        });

        notification_dispatch
            .expect_show_notification()
            .times(1)
            .return_const(());

        manager.execute_triggers(
            &REQUESTED_USER_ID,
            "".to_owned(),
            empty_update().as_ref(),
            &notification_dispatch,
        );
    }

    #[test]
    pub fn execute_triggers_when_trigger_requestable_and_requested_executes_trigger() {
        let mut notification_dispatch = MockNotificationDispatch::new();
        let mut manager = TriggerManager::default();
        manager.add_trigger(Trigger {
            requestable: true,
            requested_users: hashmap!(REQUESTED_USER_ID.clone() => BehaviorOnTrigger::NoAction),
            ..default_test_trigger()
        });

        notification_dispatch
            .expect_show_notification()
            .times(1)
            .return_const(());

        manager.execute_triggers(
            &REQUESTED_USER_ID,
            "".to_owned(),
            empty_update().as_ref(),
            &notification_dispatch,
        );
    }

    #[test]
    pub fn execute_triggers_when_trigger_requestable_and_not_requested_skips_trigger() {
        let mut notification_dispatch = MockNotificationDispatch::new();
        let mut manager = TriggerManager::default();
        manager.add_trigger(Trigger {
            requestable: true,
            requested_users: hashmap!(),
            ..default_test_trigger()
        });

        notification_dispatch
            .expect_show_notification()
            .times(0)
            .return_const(());

        manager.execute_triggers(
            &REQUESTED_USER_ID,
            "".to_owned(),
            empty_update().as_ref(),
            &notification_dispatch,
        );
    }

    #[test]
    pub fn execute_triggers_when_requested_trigger_criteria_not_met_does_not_unrequest_trigger() {
        let notification_dispatch = MockNotificationDispatch::new();
        let mut manager = TriggerManager::default();
        manager.add_trigger(Trigger {
            requestable: true,
            requested_users: hashmap!(REQUESTED_USER_ID.clone() => BehaviorOnTrigger::Remove),
            criteria: Expression::Equals(
                ValuePointer::ConstBool(false),
                ValuePointer::ConstBool(true),
            ),
            ..default_test_trigger()
        });

        manager.execute_triggers(
            &REQUESTED_USER_ID,
            "".to_owned(),
            empty_update().as_ref(),
            &notification_dispatch,
        );

        assert_eq!(
            1,
            manager
                .triggers_iter_mut()
                .next()
                .unwrap()
                .1
                .requested_users
                .len()
        );
    }

    fn default_test_trigger() -> Trigger {
        Trigger {
            name: "test trigger".to_owned(),
            enabled: true,
            requestable: true,
            requested_users: hashmap!(REQUESTED_USER_ID.clone() => BehaviorOnTrigger::NoAction),
            source: TriggerSource::AppDefaults,
            criteria: Expression::True,
            actions: vec![Action::ShowNotification(NotificationTemplate::new(
                "summary".to_owned(),
                "body".to_owned(),
            ))],
        }
    }

    fn empty_update() -> Update<SensorOutputs> {
        Update::new(empty_sensor_outputs(), empty_sensor_outputs())
    }

    fn empty_sensor_outputs() -> SensorOutputs {
        SensorOutputs { outputs: vec![] }
    }
}
