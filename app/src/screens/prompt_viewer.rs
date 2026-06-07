//! The system-prompt viewer/editor for a character (`PROMPT-9`, `PROMPT-10`,
//! `PROMPT-11`). Shows the fully-assembled prompt broken into named sections,
//! each labeled locked/editable with its source and an estimated token count.

use dioxus::prelude::*;

use lib_soulfire::ids::CharacterId;
use lib_soulfire::strings::CharacterPrompt;

use soulfire_core::ai::registry::estimate_tokens;
use soulfire_core::prompt::section::SectionSource;
use soulfire_core::prompt::{CharacterPromptInput, build_character_prompt};

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

use super::Page;

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
