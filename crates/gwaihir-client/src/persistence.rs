use crate::triggers::{TriggerManager, TriggerManagerV1, VersionedTriggerManager};
use gwaihir_client_lib::UniqueUserId;
use pro_serde_versioned::{Upgrade, VersionedUpgrade};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone)]
#[serde(into = "VersionedPersistence", from = "VersionedPersistence")]
pub struct Persistence {
    pub ignored_users: HashSet<UniqueUserId>,
    pub spacetimedb_db_name: String,

    #[serde(default)]
    pub trigger_manager: TriggerManager,
}

#[derive(Serialize, Deserialize, VersionedUpgrade, Clone)]
pub enum VersionedPersistence {
    V1(PersistenceV1),
    V2(PersistenceV2),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistenceV1 {
    pub ignored_users: HashSet<UniqueUserId>,
    pub spacetimedb_db_name: String,

    #[serde(default)]
    pub trigger_manager: TriggerManagerV1,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PersistenceV2 {
    pub ignored_users: HashSet<UniqueUserId>,
    pub spacetimedb_db_name: String,
    pub trigger_manager: TriggerManager,
}

impl Upgrade<PersistenceV2> for PersistenceV1 {
    fn upgrade(self) -> PersistenceV2 {
        PersistenceV2 {
            ignored_users: self.ignored_users,
            spacetimedb_db_name: self.spacetimedb_db_name,
            trigger_manager: VersionedTriggerManager::V1(self.trigger_manager).into(),
        }
    }
}

impl From<Persistence> for VersionedPersistence {
    fn from(value: Persistence) -> Self {
        VersionedPersistence::V2(PersistenceV2 {
            ignored_users: value.ignored_users,
            spacetimedb_db_name: value.spacetimedb_db_name,
            trigger_manager: value.trigger_manager,
        })
    }
}

impl From<VersionedPersistence> for Persistence {
    fn from(value: VersionedPersistence) -> Self {
        let upgraded = value.upgrade_to_latest();
        Persistence {
            ignored_users: upgraded.ignored_users,
            spacetimedb_db_name: upgraded.spacetimedb_db_name,
            trigger_manager: upgraded.trigger_manager,
        }
    }
}

impl Persistence {
    pub const STORAGE_KEY: &'static str = eframe::APP_KEY;
}

impl Default for Persistence {
    fn default() -> Self {
        Self {
            spacetimedb_db_name: "gwaihir-test".to_string(),
            ignored_users: Default::default(),
            trigger_manager: Default::default(),
        }
    }
}
