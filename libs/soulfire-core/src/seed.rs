//! Bundled starter worlds and first-run seeding (`ONB-5`, `DATA-21`, `DATA-24`).
//!
//! Starter worlds ship as explicit data and are seeded as ordinary, fully
//! editable/deletable `WorldBlueprint` rows on first launch. A per-starter stable
//! `seed_id` and the `InstallState` ledger ensure re-seeding never duplicates or
//! resurrects a deleted starter.

use lib_soulfire::images::{ImageTransform, WorldImage};
use lib_soulfire::install::StarterSeedRecord;
use lib_soulfire::strings::{WorldDescription, WorldPrompt, WorldTitle};
use lib_soulfire::world::WorldBlueprint;

use crate::clock::Clock;
use crate::error::CoreResult;
use crate::store::Store;

/// The starter-catalog seed version; bump when the shipped catalog changes
/// (`DATA-24`).
pub const STARTER_SEED_VERSION: u32 = 1;

/// One shipped starter world (`DATA-21`).
pub struct StarterWorld {
    pub seed_id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub world_prompt: &'static str,
    pub image: WorldImage,
}

/// The shipped starter catalog. `beneath_verath` is the lead starter (`ONB-5`).
pub const STARTERS: &[StarterWorld] = &[StarterWorld {
    seed_id: "beneath_verath",
    title: "Beneath Verath",
    description: "A drowned city stirs beneath the tide. You wake on its black shore with a debt you cannot remember and a name the water keeps whispering.",
    image: WorldImage::EmojiWater,
    world_prompt: BENEATH_VERATH_PROMPT,
}];

const BENEATH_VERATH_PROMPT: &str = r#"# World Blueprint: Beneath Verath

## Setting
- Verath was a great harbour-city of bridges, bells, and lantern-lit canals, sunk in a single night a generation ago when the sea rose without storm or warning.
- The drowned city did not die. Its lower districts breathe under the water; air pockets, sealed halls, and stubborn lanterns keep a half-life going in the deep.
- A handful of survivors and scavengers live on the broken upper terraces and the tide-line shanties of Saltreach, the only dry ground left.
- The water remembers. People who drink too deeply of Verath's wells begin to hear a voice — patient, oceanic — that calls itself the Tide-Mother and speaks of a bargain unpaid.
- Hard rules of this world: no firearms or modern technology; magic is rare, costly, and tied to water, salt, and memory; the deep city is lightless and pressured, survivable only via the old sealed ways and the air-bells the drowned guilds left behind.

## The player
- You are a newcomer washed up on Saltreach's shore with no possessions and a fragment of memory: a debt, a drowned name, and the certainty that you have been to Verath before. The intro should establish this and prompt the first action; do not reveal the truth of the debt yet.

## Act 0 — Saltreach (current act = 0)
- Find your feet among the scavengers. Earn trust, a meal, and a way down into the shallows.
- Key NPCs the player may meet here (do not introduce others until the story warrants):
  - **Mooring Kell** — a weathered tide-runner who ferries scavengers to the safe wrecks; gruff, fair, owes no one.
  - **The Lampwright** — keeps Saltreach's beacon lit; knows which sealed ways still hold air and which have flooded.
  - **Sera of the Salt** — a young diver with a relic she cannot read and a sibling lost to the deep.
- Milestone: secure a guide and a working air-bell, and learn the first rumour of the Tide-Mother's bargain.

## Act I — The Shallows
- Descend into Verath's upper drowned districts. Recover what scavengers prize; uncover that the flood was no accident.
- Soft consequences: the deeper you go, the more the water's voice grows; choices about what you take and whom you trust shape who will help you later.
- Milestone: reach the Bell of the Drowned Guild and hear the Tide-Mother name your debt.

## Act II — The Deep City
- Cross the lightless lower city using the sealed ways. Confront what the old harbour-masters bargained away, and who paid the price.
- Milestone: stand before the Tide-Mother and choose whether to honour, break, or rewrite the bargain.

## Act III — The Turning Tide
- The consequence of your choice reshapes Verath: the city rises, drowns further, or finds a third fate. Saltreach, Kell, the Lampwright, and Sera react according to all that came before.
- Final milestone: settle your debt with the deep, for good or ill, and decide what Verath becomes.

## Tone & guidance
- Atmospheric, melancholy, salt-and-lantern fantasy. The horror is quiet and oceanic, not gory.
- NPCs have their own lives and agendas and remember how the player treats them.
- Keep the truth of the debt and the Tide-Mother's nature hidden until the story earns the reveal.
"#;

impl StarterWorld {
    /// Build a fresh blueprint for this starter (`DATA-21`).
    fn to_blueprint(&self, clock: &dyn Clock) -> WorldBlueprint {
        let now = clock.now();
        WorldBlueprint::builder()
            .title(WorldTitle::coerce(self.title))
            .description(WorldDescription::coerce(self.description))
            .world_prompt(WorldPrompt::coerce(self.world_prompt))
            .image(self.image)
            .image_transform(ImageTransform::default())
            .created_at(now)
            .updated_at(now)
            .build()
    }
}

/// Seed any starter worlds not already handled, recording each in the ledger
/// (`ONB-5`). Idempotent: a starter that was ever seeded — even if since deleted
/// — is never re-created.
pub fn seed_starter_worlds(store: &Store, clock: &dyn Clock) -> CoreResult<()> {
    let mut install = store.install_state()?;
    let mut seeded_any = false;

    for starter in STARTERS {
        if install.starter_already_handled(starter.seed_id) {
            continue;
        }
        let blueprint = starter.to_blueprint(clock);
        store.save_blueprint(&blueprint)?;
        install.starter_worlds.insert(
            starter.seed_id.to_string(),
            StarterSeedRecord {
                blueprint_id: Some(blueprint.blueprint_id.clone()),
                seeded_at: Some(clock.now()),
                deleted: false,
            },
        );
        seeded_any = true;
    }

    if seeded_any || install.starter_seed_version != STARTER_SEED_VERSION {
        install.starter_seed_version = STARTER_SEED_VERSION;
        store.save_install_state(&install)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::MockClock;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::initialize(dir.path(), "pw").unwrap();
        (dir, store)
    }

    #[test]
    fn seeds_starters_as_editable_blueprints_with_ledger() {
        // AC-ONB-d: starter worlds appear as blueprints with stable seed ids.
        let (_dir, store) = store();
        let clock = MockClock::at_epoch();
        seed_starter_worlds(&store, &clock).unwrap();
        assert_eq!(store.count_blueprints().unwrap(), STARTERS.len() as i64);
        let install = store.install_state().unwrap();
        let rec = install.starter("beneath_verath").unwrap();
        assert!(rec.was_seeded());
        assert!(rec.blueprint_id.is_some());
        // The lead starter is present and editable like any blueprint.
        let bp = store.blueprint(rec.blueprint_id.as_ref().unwrap()).unwrap().unwrap();
        assert_eq!(bp.title.as_str(), "Beneath Verath");
    }

    #[test]
    fn reseeding_does_not_duplicate() {
        let (_dir, store) = store();
        let clock = MockClock::at_epoch();
        seed_starter_worlds(&store, &clock).unwrap();
        seed_starter_worlds(&store, &clock).unwrap();
        assert_eq!(store.count_blueprints().unwrap(), STARTERS.len() as i64);
    }

    #[test]
    fn deleted_starter_is_not_resurrected() {
        // AC-ONB-d: deleting a starter and relaunching does not bring it back.
        let (_dir, store) = store();
        let clock = MockClock::at_epoch();
        seed_starter_worlds(&store, &clock).unwrap();
        let id = store
            .install_state()
            .unwrap()
            .starter("beneath_verath")
            .unwrap()
            .blueprint_id
            .clone()
            .unwrap();
        store.delete_blueprint(&id).unwrap();
        // Re-seed: must not re-create the deleted starter.
        seed_starter_worlds(&store, &clock).unwrap();
        assert_eq!(store.count_blueprints().unwrap(), 0);
    }
}
