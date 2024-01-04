use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use derive_new::new;
use gwaihir_client_lib::UniqueUserId;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct ChangeMatcher {
    matchers: Vec<Matcher>,
}

#[derive(new, Serialize, Deserialize)]
pub struct Matcher {
    pub criteria: MatchCriteria,
    pub drop_after_match: bool,
}

#[derive(new)]
pub struct Update<T> {
    original: T,
    updated: T,
}

impl ChangeMatcher {
    pub fn match_once_when_online(&mut self, user_id: UniqueUserId) {
        self.add_match_once(MatchCriteria::UserComesOnline(user_id));
    }

    pub fn remove_matcher(&mut self, predicate: impl Fn(&Matcher) -> bool) {
        self.matchers.retain(|c| !predicate(c));
    }

    pub fn add_match_once(&mut self, criteria: MatchCriteria) {
        self.matchers.push(Matcher::new(criteria, true));
    }

    pub fn add_match(&mut self, criteria: MatchCriteria) {
        self.matchers.push(Matcher::new(criteria, false));
    }

    pub fn get_matches(
        &mut self,
        user_id: &UniqueUserId,
        update: Update<&SensorOutputs>,
    ) -> Vec<MatchCriteria> {
        let mut matched: Vec<MatchCriteria> = Vec::new();

        self.matchers.retain(|el| {
            let matches = el.criteria.matches(user_id, &update);
            if matches {
                matched.push(el.criteria.clone());
            }

            !el.drop_after_match || !matches
        });

        matched
    }

    pub fn has_matcher(&self, predicate: impl Fn(&Matcher) -> bool) -> bool {
        self.matchers.iter().any(predicate)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MatchCriteria {
    UserComesOnline(UniqueUserId),
}

impl MatchCriteria {
    pub fn matches(&self, user_id: &UniqueUserId, update: &Update<&SensorOutputs>) -> bool {
        match self {
            MatchCriteria::UserComesOnline(match_user_id) => {
                if match_user_id != user_id {
                    return false;
                }

                let original_online_status = update.original.get_online_status();
                let updated_online_status = update.updated.get_online_status();
                if let Some(original_online_status) = original_online_status {
                    if let Some(updated_online_status) = updated_online_status {
                        return !original_online_status.online && updated_online_status.online;
                    }
                }

                false
            }
        }
    }
}
