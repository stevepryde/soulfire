//! First-run and seed bookkeeping (`DATA-24`, `ONB-5`). One row. Prevents
//! first-run auto-start and starter seeding from recurring.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use sp_core::datetime::SpDateTime;

use crate::ids::WorldBlueprintId;

/// One entry in the starter-worlds ledger, keyed by a stable starter seed id
/// (`DATA-24`, `ONB-5`). Records which blueprint was created for that starter and
/// whether the user later deleted it (so re-seeding never resurrects it).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StarterSeedRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blueprint_id: Option<WorldBlueprintId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seeded_at: Option<SpDateTime>,
    /// True once the user has deleted the blueprint that was seeded for this
    /// starter; a deleted starter is never re-created (`ONB-5`).
    #[serde(default)]
    pub deleted: bool,
}

impl StarterSeedRecord {
    /// True if this starter has been seeded at least once (regardless of whether
    /// it was later deleted).
    pub fn was_seeded(&self) -> bool {
        self.seeded_at.is_some()
    }
}

/// First-run and seed bookkeeping (one row, `DATA-24`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, bon::Builder)]
pub struct InstallState {
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    /// True once the first-run flow has completed (auto-start does not recur,
    /// `ONB-4`).
    #[builder(default)]
    #[serde(default)]
    pub first_run_completed: bool,
    /// The starter-catalog seed version most recently applied (`DATA-24`).
    #[builder(default)]
    #[serde(default)]
    pub starter_seed_version: u32,
    /// The starter-worlds ledger, keyed by stable starter seed id (`DATA-24`).
    #[builder(default)]
    #[serde(default)]
    pub starter_worlds: IndexMap<String, StarterSeedRecord>,
}

impl InstallState {
    /// The ledger record for a starter seed id, if any.
    pub fn starter(&self, seed_id: &str) -> Option<&StarterSeedRecord> {
        self.starter_worlds.get(seed_id)
    }

    /// True if a starter has ever been seeded (so it must not be seeded again,
    /// even if the user has since deleted it, `ONB-5`).
    pub fn starter_already_handled(&self, seed_id: &str) -> bool {
        self.starter_worlds
            .get(seed_id)
            .map(StarterSeedRecord::was_seeded)
            .unwrap_or(false)
    }
}

fn one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_install_state_defaults() {
        // DATA-24: first_run_completed=false, starter_seed_version=0, empty ledger.
        let s = InstallState::default();
        assert!(!s.first_run_completed);
        assert_eq!(s.starter_seed_version, 0);
        assert!(s.starter_worlds.is_empty());
    }

    #[test]
    fn seeded_starter_is_handled_even_after_deletion() {
        let mut s = InstallState::default();
        s.starter_worlds.insert(
            "beneath_verath".to_string(),
            StarterSeedRecord {
                blueprint_id: Some(WorldBlueprintId::new()),
                seeded_at: Some(SpDateTime::now()),
                deleted: true,
            },
        );
        assert!(s.starter_already_handled("beneath_verath"));
        assert!(!s.starter_already_handled("unseen_starter"));
    }
}
