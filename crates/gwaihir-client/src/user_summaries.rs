use std::collections::HashMap;

use gwaihir_client_lib::UniqueUserId;

pub struct UserSummaries {
    summaries: HashMap<UniqueUserId, String>,
}

impl UserSummaries {
    pub fn new() -> Self {
        Self {
            summaries: HashMap::new(),
        }
    }

    pub fn get(&self, id: &UniqueUserId) -> Option<String> {
        self.summaries.get(id).map(|s| s.to_owned())
    }

    pub fn set_summary(&mut self, id: UniqueUserId, summary: String) {
        self.summaries.insert(id, summary);
    }

    pub fn clear_summary(&mut self, id: &UniqueUserId) {
        self.summaries.remove(id);
    }
}

impl Default for UserSummaries {
    fn default() -> Self {
        Self::new()
    }
}
