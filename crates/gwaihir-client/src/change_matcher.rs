use crate::sensors::outputs::sensor_outputs::SensorOutputs;
use gwaihir_client_lib::UniqueUserId;

#[derive(Default)]
pub struct ChangeMatcher {
    match_once: Vec<MatchCriteria>,
}

pub struct Update<T> {
    original: T,
    updated: T,
}

impl<T> Update<T> {
    pub fn new(original: T, updated: T) -> Self {
        Update { original, updated }
    }
}

impl ChangeMatcher {
    pub fn match_once_when_online(&mut self, user_id: UniqueUserId) {
        self.add_match_once(MatchCriteria::UserComesOnline(user_id));
    }

    pub fn remove_match_once(&mut self, predicate: impl Fn(&MatchCriteria) -> bool) {
        self.match_once.retain(|c| !predicate(c));
    }

    pub fn add_match_once(&mut self, criteria: MatchCriteria) {
        self.match_once.push(criteria);
    }

    pub fn get_matches(
        &mut self,
        user_id: &UniqueUserId,
        update: Update<&SensorOutputs>,
    ) -> Vec<MatchCriteria> {
        let mut matched: Vec<MatchCriteria> = Vec::new();

        self.match_once.retain(|el| {
            let matches = el.matches(user_id, &update);
            if matches {
                matched.push(el.clone());
            }

            !matches
        });

        matched
    }

    pub fn has_criteria_once(&self, predicate: impl Fn(&MatchCriteria) -> bool) -> bool {
        self.match_once.iter().any(predicate)
    }
}

#[derive(Clone)]
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
