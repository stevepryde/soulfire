//! The system-prompt viewer/editor for a character (`PROMPT-9`, `PROMPT-10`,
//! `PROMPT-11`). Shows the fully-assembled prompt broken into named sections,
//! each labeled locked/editable with its source and an estimated token count.

use dioxus::prelude::*;

use lib_soulfire::ids::{AdventureId, CharacterId};
use lib_soulfire::strings::CharacterPrompt;

use soulfire_core::ai::registry::estimate_tokens;
use soulfire_core::prompt::section::SectionSource;
use soulfire_core::prompt::{CharacterPromptInput, build_character_prompt};
use soulfire_core::world::prompts as wp;

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

use super::Page;

/// The prompt viewer for an adventure (`PROMPT-9`): the game-master instructions
/// plus the live context that would be sent for the next turn, each labeled and
/// token-estimated. The world prompt is this adventure's private copy, changed
/// only via `/gm` (`PROMPT-10`, `WORLD-17`).
#[component]
pub fn AdventurePromptViewer(adventure_id: AdventureId) -> Element {
    data::subscribe();
    let app = current_app();
    let Some(adv) = app.store.adventure(&adventure_id).ok().flatten() else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "Adventure not found." } };
    };
    let settings = app.store.app_settings().unwrap_or_default();
    let adult = settings.content_toggles.adult_content;
    let ext = app
        .store
        .player_profile()
        .unwrap_or_default()
        .prompt_extension
        .map(|p| p.to_string());

    let instructions = wp::narrative_instructions(adv.world_prompt.as_str(), ext.as_deref(), adult);
    let input = wp::narrative_input(
        adv.significant_events.as_str(),
        adv.adventure_state.as_str(),
        adv.story_summary.as_str(),
        adv.recent_summary.as_str(),
        adv.previous_narrative.as_deref().unwrap_or(""),
        "<your next action>",
    );
    let total: usize = estimate_tokens(&instructions)
        + input
            .iter()
            .map(|m| estimate_tokens(&m.content))
            .sum::<usize>();

    rsx! {
        Page { title: "Adventure Prompt".to_string(),
            div { class: "flex items-center justify-between mb-4",
                p { class: "text-secondary-text", "What is sent to the game master for your next turn" }
                span { class: "text-sm text-primary font-mono", "≈ {total} tokens" }
            }
            PromptCard {
                header: "Game Master Instructions".to_string(),
                locked: true,
                note: "Locked. The world prompt within is this adventure's private copy — change it via /gm.".to_string(),
                body: instructions,
            }
            for msg in input.clone() {
                {
                    let (header, body) = split_header(&msg.content);
                    rsx! {
                        PromptCard {
                            header,
                            locked: true,
                            note: "Live context".to_string(),
                            body,
                        }
                    }
                }
            }
            button {
                class: "mt-2 text-secondary-text text-sm hover:underline",
                onclick: move |_| navigate(Screen::Play(adventure_id.clone())),
                "← Back to adventure"
            }
        }
    }
}

/// Split a `# Header…\nbody` message into its header line and body.
fn split_header(content: &str) -> (String, String) {
    match content.split_once('\n') {
        Some((h, b)) => (h.trim_start_matches('#').trim().to_string(), b.to_string()),
        None => (content.to_string(), String::new()),
    }
}

#[component]
fn PromptCard(header: String, locked: bool, note: String, body: String) -> Element {
    let tokens = estimate_tokens(&body);
    rsx! {
        div { class: "bg-surface border border-border rounded-xl mb-3 overflow-hidden",
            div { class: "flex items-center justify-between px-4 py-2 border-b border-border",
                div { class: "flex items-center gap-2",
                    span { class: "font-semibold text-primary-text", "{header}" }
                    if locked {
                        span { class: "text-[10px] uppercase bg-gray-700 text-gray-300 px-1.5 py-0.5 rounded", "Locked" }
                    }
                }
                span { class: "text-xs text-secondary-text font-mono", "≈ {tokens}" }
            }
            p { class: "px-4 pt-1 text-[11px] text-secondary-text", "{note}" }
            div { class: "p-4 text-sm text-secondary-text whitespace-pre-wrap font-mono max-h-48 overflow-y-auto scrollbar-premium", "{body}" }
        }
    }
}

#[component]
pub fn PromptViewer(character_id: CharacterId) -> Element {
    data::subscribe();
    let app = current_app();
    let Some(character) = app.store.character(&character_id).ok().flatten() else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "Character not found." } };
    };
    let settings = app.store.app_settings().unwrap_or_default();

    // Gather owned context strings (they must outlive the borrowed input).
    let world_context = character
        .source_blueprint_id
        .as_ref()
        .and_then(|b| app.store.blueprint(b).ok().flatten())
        .map(|b| b.world_prompt.to_string());
    let adventure = character
        .source_adventure_id
        .as_ref()
        .and_then(|a| app.store.adventure(a).ok().flatten());
    let world_state = adventure.as_ref().map(|a| a.adventure_state.to_string());
    let story = adventure
        .as_ref()
        .map(|a| a.story_summary.to_string())
        .filter(|s| !s.is_empty());
    let extracted = character.extracted_context.as_ref().map(|c| c.to_string());
    let dynamic = character.character_state.as_ref().map(|c| c.to_string());
    let prompt_text = character.prompt.to_string();

    let input = CharacterPromptInput {
        character_prompt: &prompt_text,
        extracted_context: extracted.as_deref(),
        character_state: dynamic.as_deref(),
        is_adventure_linked: character.source_adventure_id.is_some(),
        world_context: world_context.as_deref(),
        world_state: world_state.as_deref(),
        story_so_far: story.as_deref(),
        toggles: settings.content_toggles,
    };
    let assembled = build_character_prompt(&input);
    let total_tokens = estimate_tokens(&assembled.instructions());

    let mut editing = use_signal(|| false);
    let mut draft = use_signal(|| prompt_text.clone());

    rsx! {
        Page { title: "System Prompt".to_string(),
            div { class: "flex items-center justify-between mb-4",
                p { class: "text-secondary-text", "{character.name} — what is sent for the next turn" }
                span { class: "text-sm text-primary font-mono", "≈ {total_tokens} tokens" }
            }
            for section in assembled.sections.clone() {
                {
                    let app = app.clone();
                    let character = character.clone();
                    let header = section.header.clone();
                    let body = section.body.clone();
                    let tokens = estimate_tokens(&section.rendered());
                    let is_authored = section.source == SectionSource::AuthoredCharacterPrompt;
                    rsx! {
                        div { class: "bg-surface border border-border rounded-xl mb-3 overflow-hidden",
                            div { class: "flex items-center justify-between px-4 py-2 border-b border-border",
                                div { class: "flex items-center gap-2",
                                    span { class: "font-semibold text-primary-text", "{header}" }
                                    if section.locked {
                                        span { class: "text-[10px] uppercase bg-gray-700 text-gray-300 px-1.5 py-0.5 rounded", "Locked" }
                                    } else {
                                        span { class: "text-[10px] uppercase bg-primary/30 text-primary px-1.5 py-0.5 rounded", "Editable" }
                                    }
                                }
                                span { class: "text-xs text-secondary-text font-mono", "≈ {tokens}" }
                            }
                            if is_authored && editing() {
                                div { class: "p-4",
                                    textarea {
                                        class: "input-premium w-full font-mono text-sm",
                                        rows: "10",
                                        value: "{draft}",
                                        oninput: move |e| draft.set(e.value()),
                                    }
                                    div { class: "flex gap-2 mt-2",
                                        button {
                                            class: "crm-primary-button px-4 py-1.5 rounded-lg text-sm",
                                            onclick: move |_| {
                                                let mut c = character.clone();
                                                c.prompt = CharacterPrompt::coerce(&draft());
                                                c.updated_at = sp_core::datetime::SpDateTime::now();
                                                let _ = app.store.save_character(&c);
                                                editing.set(false);
                                                data::touch();
                                            },
                                            "Save"
                                        }
                                        button {
                                            class: "px-4 py-1.5 rounded-lg text-sm border border-border text-secondary-text",
                                            onclick: move |_| editing.set(false),
                                            "Cancel"
                                        }
                                    }
                                }
                            } else {
                                div {
                                    class: "p-4 text-sm text-secondary-text whitespace-pre-wrap font-mono max-h-48 overflow-y-auto scrollbar-premium",
                                    onclick: move |_| if is_authored { editing.set(true) },
                                    "{body}"
                                }
                            }
                        }
                    }
                }
            }
            button {
                class: "mt-2 text-secondary-text text-sm hover:underline",
                onclick: move |_| navigate(Screen::Characters),
                "← Back to characters"
            }
        }
    }
}
