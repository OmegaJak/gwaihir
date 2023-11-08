use linked_hash_set::LinkedHashSet;
use nutype::nutype;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::active_window_provider::WindowIdentifiers;

#[nutype(derive(Serialize, Deserialize, AsRef, Clone, Into, Debug, Hash, PartialEq, Eq))]
pub struct AppName(String);

#[nutype(derive(Serialize, Deserialize, AsRef, Clone, Into, Debug, Hash, PartialEq, Eq))]
pub struct OriginalTitle(String);

#[nutype(derive(Serialize, Deserialize, AsRef, Clone, Into, Debug))]
pub struct MappedTitle(String);

#[derive(Serialize, Deserialize)]
pub struct AcceptedMapping {
    accepted_examples: HashMap<OriginalTitle, MappedTitle>,

    #[serde(with = "serde_regex_as_str")]
    regex: Regex,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WindowTitleMappings {
    mappings: HashMap<AppName, AcceptedMapping>,
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

mod serde_regex_as_str {
    use regex::Regex;

    pub fn serialize<S>(regex: &Regex, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str(regex.as_str())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Regex, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::de::Deserialize::deserialize(d)?;
        Regex::new(s).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
