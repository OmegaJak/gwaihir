use linked_hash_set::LinkedHashSet;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

use super::active_window_provider::WindowIdentifiers;

#[derive(Serialize, Deserialize, Default)]
pub struct WindowTitleMappings {
    // TODO
}

impl WindowTitleMappings {
    pub fn map(&self, window_identifiers: WindowIdentifiers) -> String {
        window_identifiers.app_name
    }
}

pub struct WindowTitleMapper {
    accepted_mappings: Arc<RwLock<WindowTitleMappings>>,
    pending_mappings: LinkedHashSet<WindowIdentifiers>,
}

impl WindowTitleMapper {
    pub fn new(accepted_mappings: Arc<RwLock<WindowTitleMappings>>) -> Self {
        Self {
            accepted_mappings,
            pending_mappings: LinkedHashSet::new(),
        }
    }

    pub fn record_observed_title(&mut self, window_identifiers: WindowIdentifiers) {
        self.pending_mappings.insert(window_identifiers);
    }

    pub fn iter_pending(&self) -> linked_hash_set::Iter<'_, WindowIdentifiers> {
        self.pending_mappings.iter()
    }
}
