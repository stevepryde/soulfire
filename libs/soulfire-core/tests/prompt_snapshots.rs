//! Full prompt hash snapshots for representative OG parity fixtures (TEST-10).
//!
//! The checked fixture stores byte length + SHA-256 for each rendered prompt. A
//! hash change means the complete prompt changed, while anchors keep the fixture
//! readable and tied to the behavior being protected.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use soulfire_core::character::prompts as character_prompts;
use soulfire_core::chat::prompts as chat_prompts;
use soulfire_core::image::{character_portrait_prompt, world_cover_prompt};
use soulfire_core::model::settings::ContentToggles;
use soulfire_core::prompt::{CharacterPromptInput, build_character_prompt};
use soulfire_core::world::builder as world_builder;
use soulfire_core::world::prompts as world_prompts;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Snapshot {
    name: String,
    bytes: usize,
    sha256: String,
    anchors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExpectedSnapshots {
    snapshots: Vec<Snapshot>,
}

fn world_prompt() -> &'static str {
    "## World\nVerath is a flooded city where bells remember old promises.\n## Rules\nConserve keys, lanterns, tides, and promises."
}

fn adventure_state() -> &'static str {
    "{\"player\":{\"name\":\"Diver\"},\"current_situation\":{\"location\":\"bell tower\",\"time\":\"dusk\",\"day\":2},\"gm_notes\":[\"Have the seventh bell answer if the lantern key is raised.\"]}"
}

fn story_summary() -> &'static str {
    "## Rolling Story\nThe player entered Verath and found the lantern key.\n\n## Recent Turns\n1. Diver raised the key and Lyra listened for the bells."
}

fn recent_events() -> &'static str {
    "Bell tower: Lyra is beside the player at the tide line."
}

fn significant_events() -> &'static str {
    "evt_1 (weight 5): (Day 2, Bell Tower) The tide receded from the bell tower."
}

fn previous_narrative() -> &'static str {
    "The tower door opened under the tide line."
}

fn messages_snapshot(messages: &[soulfire_core::ai::types::PromptMessage]) -> String {
    serde_json::to_string_pretty(messages).unwrap()
}

fn hash(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn snapshot(name: &str, text: String, anchors: &[&str]) -> Snapshot {
    Snapshot {
        name: name.to_string(),
        bytes: text.len(),
        sha256: hash(&text),
        anchors: anchors.iter().map(|s| s.to_string()).collect(),
    }
}

fn actual_snapshots() -> Vec<Snapshot> {
    let character_chat = build_character_prompt(&CharacterPromptInput {
        character_prompt: "You are Lyra, a calm guide who remembers the player's choices.",
        extracted_context: Some("Lyra was met inside the drowned city of Verath."),
        character_state: Some("Lyra trusts the player and is watching the tides."),
        is_adventure_linked: true,
        world_context: Some(world_prompt()),
        world_state: Some(adventure_state()),
        story_so_far: Some(story_summary()),
        toggles: ContentToggles {
            adult_content: true,
        },
    })
    .instructions();

    let character_builder_input = character_prompts::builder_input(
        "Lyra",
        "Moonlit guide",
        "A serene lantern keeper.",
        "You are Lyra, a calm guide.",
        "Welcome back, traveler.",
        "User: Make Lyra more watchful.\nAssistant: I made Lyra quieter and more observant.",
        "Give her a stronger connection to the seventh bell.",
    );

    vec![
        snapshot(
            "character_chat_world_linked_instructions",
            character_chat,
            &[
                "## World Context",
                "## How to Be This Character",
                "## Your Current State",
            ],
        ),
        snapshot(
            "chat_summary_prompt",
            chat_prompts::summary_prompt(
                Some("Old summary."),
                "Diver: I found the lantern key.\nLyra: Then the bell tower may answer us.",
            ),
            &[
                "Summarize this conversation",
                "Previous summary:",
                "New messages:",
            ],
        ),
        snapshot(
            "chat_state_update_prompt",
            chat_prompts::state_update_prompt(
                "Lyra",
                "Lyra: a steadfast guide.",
                "Calm and watchful.",
                "Diver: I found the lantern key.\nLyra: Then the bell tower may answer us.",
            ),
            &[
                "## Character Profile",
                "## Current Dynamic State",
                "Return ONLY",
            ],
        ),
        snapshot(
            "character_builder_instructions",
            character_prompts::builder_instructions(),
            &[
                "collaborative character builder",
                "\"assistant_message\"",
                "description is only a compact",
            ],
        ),
        snapshot(
            "character_builder_input",
            character_builder_input,
            &[
                "Current character:",
                "Recent builder chat:",
                "Latest user message:",
            ],
        ),
        snapshot(
            "npc_extraction_persona_prompt",
            character_prompts::extraction_system_prompt(
                world_prompt(),
                adventure_state(),
                story_summary(),
                "Lyra",
            ),
            &[
                "extracting an NPC",
                "## World Context",
                "### Voice & Speaking Style",
            ],
        ),
        snapshot(
            "npc_extraction_initial_state_prompt",
            character_prompts::initial_state_system_prompt(
                "Lyra",
                "Lyra: a steadfast guide.",
                story_summary(),
            ),
            &[
                "initial dynamic state",
                "### Current Emotional State",
                "roughly 300-500 words",
            ],
        ),
        snapshot(
            "world_builder_instructions",
            world_builder::builder_instructions(),
            &[
                "collaborative world builder",
                "\"world_prompt\"",
                "full replacement world prompt",
            ],
        ),
        snapshot(
            "world_intro_instructions",
            world_prompts::intro_narrative_instructions(world_prompt()),
            &[
                "Write an engaging introduction paragraph",
                "# World Blueprint",
                "first action",
            ],
        ),
        snapshot(
            "world_initial_state_instructions",
            world_prompts::initial_state_instructions(world_prompt()),
            &[
                "creating an initial adventure state",
                "Current situation",
                "GM Notes",
            ],
        ),
        snapshot(
            "world_narrative_instructions",
            world_prompts::narrative_instructions(
                world_prompt(),
                Some("Let tactile bell imagery matter."),
                true,
            ),
            &[
                "# Your task:",
                "## Mature roleplay",
                "# Additional Directives:",
            ],
        ),
        snapshot(
            "world_narrative_input_messages",
            messages_snapshot(&world_prompts::narrative_input(
                significant_events(),
                adventure_state(),
                story_summary(),
                recent_events(),
                previous_narrative(),
                "Raise the lantern key.",
            )),
            &[
                "Significant Events",
                "Current Adventure State",
                "Player's Action",
            ],
        ),
        snapshot(
            "world_diff_state_instructions",
            world_prompts::diff_state_update_instructions(world_prompt(), "", true),
            &["\"patches\"", "Conservation", "## Mature roleplay"],
        ),
        snapshot(
            "world_full_state_instructions",
            world_prompts::full_state_update_instructions(world_prompt(), "COMPACT NOW", true),
            &["\"updated_state\"", "significant_events", "COMPACT NOW"],
        ),
        snapshot(
            "gm_classification_instructions",
            world_prompts::gm_classification_instructions(),
            &["Classify one out-of-band", "\"answer_only\"", "\"both\""],
        ),
        snapshot(
            "gm_answer_instructions",
            world_prompts::gm_answer_instructions(world_prompt()),
            &["out-of-band request", "\"response\"", "## Consent gating"],
        ),
        snapshot(
            "gm_proposal_instructions",
            world_prompts::gm_proposal_instructions(world_prompt(), "both", true),
            &[
                "# Classified request intent:\nboth",
                "\"updated_adventure_state\"",
                "## Mature roleplay",
            ],
        ),
        snapshot(
            "gm_command_input_messages",
            messages_snapshot(&world_prompts::gm_command_input(
                adventure_state(),
                "Diver: I raise the lantern key.\nGM: The bells answer.",
                recent_events(),
                significant_events(),
                story_summary(),
                previous_narrative(),
                "Make the seventh bell a permanent world rule.",
            )),
            &[
                "Recent Roleplay Messages",
                "Previous Story Narration",
                "Out-of-band GM Request",
            ],
        ),
        snapshot(
            "image_character_portrait_prompt",
            character_portrait_prompt("Lyra", "A serene lantern keeper.", "fallback prompt"),
            &["character portrait", "Lyra", "Head-and-shoulders"],
        ),
        snapshot(
            "image_world_cover_prompt",
            world_cover_prompt(
                "Beneath Verath",
                "A drowned city of secret bells.",
                world_prompt(),
            ),
            &["Wide cinematic cover art", "Beneath Verath", "no text"],
        ),
    ]
}

#[test]
fn representative_og_prompts_match_snapshots() {
    let expected: ExpectedSnapshots =
        serde_json::from_str(include_str!("fixtures/og_prompt_snapshots.json")).unwrap();
    let actual = actual_snapshots();

    if expected.snapshots.len() != actual.len() {
        println!(
            "{}",
            serde_json::to_string_pretty(&ExpectedSnapshots { snapshots: actual }).unwrap()
        );
        panic!("prompt snapshot fixture count is stale");
    }

    for (expected, actual) in expected.snapshots.iter().zip(actual.iter()) {
        assert_eq!(
            expected, actual,
            "prompt snapshot changed: {}",
            expected.name
        );
        for anchor in &expected.anchors {
            assert!(
                render_prompt_for_anchor(&expected.name).contains(anchor),
                "missing prompt anchor {anchor:?} in {}",
                expected.name
            );
        }
    }
}

fn render_prompt_for_anchor(name: &str) -> String {
    // Re-render only on the rare assertion failure path; keep Snapshot compact.
    match actual_snapshots().into_iter().find(|s| s.name == name) {
        Some(_) => {}
        None => panic!("unknown snapshot {name}"),
    }

    let text = match name {
        "character_chat_world_linked_instructions" => {
            build_character_prompt(&CharacterPromptInput {
                character_prompt: "You are Lyra, a calm guide who remembers the player's choices.",
                extracted_context: Some("Lyra was met inside the drowned city of Verath."),
                character_state: Some("Lyra trusts the player and is watching the tides."),
                is_adventure_linked: true,
                world_context: Some(world_prompt()),
                world_state: Some(adventure_state()),
                story_so_far: Some(story_summary()),
                toggles: ContentToggles {
                    adult_content: true,
                },
            })
            .instructions()
        }
        "chat_summary_prompt" => chat_prompts::summary_prompt(
            Some("Old summary."),
            "Diver: I found the lantern key.\nLyra: Then the bell tower may answer us.",
        ),
        "chat_state_update_prompt" => chat_prompts::state_update_prompt(
            "Lyra",
            "Lyra: a steadfast guide.",
            "Calm and watchful.",
            "Diver: I found the lantern key.\nLyra: Then the bell tower may answer us.",
        ),
        "character_builder_instructions" => character_prompts::builder_instructions(),
        "character_builder_input" => character_prompts::builder_input(
            "Lyra",
            "Moonlit guide",
            "A serene lantern keeper.",
            "You are Lyra, a calm guide.",
            "Welcome back, traveler.",
            "User: Make Lyra more watchful.\nAssistant: I made Lyra quieter and more observant.",
            "Give her a stronger connection to the seventh bell.",
        ),
        "npc_extraction_persona_prompt" => character_prompts::extraction_system_prompt(
            world_prompt(),
            adventure_state(),
            story_summary(),
            "Lyra",
        ),
        "npc_extraction_initial_state_prompt" => character_prompts::initial_state_system_prompt(
            "Lyra",
            "Lyra: a steadfast guide.",
            story_summary(),
        ),
        "world_builder_instructions" => world_builder::builder_instructions(),
        "world_intro_instructions" => world_prompts::intro_narrative_instructions(world_prompt()),
        "world_initial_state_instructions" => {
            world_prompts::initial_state_instructions(world_prompt())
        }
        "world_narrative_instructions" => world_prompts::narrative_instructions(
            world_prompt(),
            Some("Let tactile bell imagery matter."),
            true,
        ),
        "world_narrative_input_messages" => messages_snapshot(&world_prompts::narrative_input(
            significant_events(),
            adventure_state(),
            story_summary(),
            recent_events(),
            previous_narrative(),
            "Raise the lantern key.",
        )),
        "world_diff_state_instructions" => {
            world_prompts::diff_state_update_instructions(world_prompt(), "", true)
        }
        "world_full_state_instructions" => {
            world_prompts::full_state_update_instructions(world_prompt(), "COMPACT NOW", true)
        }
        "gm_classification_instructions" => world_prompts::gm_classification_instructions(),
        "gm_answer_instructions" => world_prompts::gm_answer_instructions(world_prompt()),
        "gm_proposal_instructions" => {
            world_prompts::gm_proposal_instructions(world_prompt(), "both", true)
        }
        "gm_command_input_messages" => messages_snapshot(&world_prompts::gm_command_input(
            adventure_state(),
            "Diver: I raise the lantern key.\nGM: The bells answer.",
            recent_events(),
            significant_events(),
            story_summary(),
            previous_narrative(),
            "Make the seventh bell a permanent world rule.",
        )),
        "image_character_portrait_prompt" => {
            character_portrait_prompt("Lyra", "A serene lantern keeper.", "fallback prompt")
        }
        "image_world_cover_prompt" => world_cover_prompt(
            "Beneath Verath",
            "A drowned city of secret bells.",
            world_prompt(),
        ),
        _ => unreachable!(),
    };
    text
}
