//! Manual editors for characters and world blueprints (`UI-18`, `CHAR-1`..`CHAR-5`,
//! `WORLD-2`).

use std::str::FromStr;

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::character::{Character, CreativityControls, InitialMessage};
use lib_soulfire::ids::{CharacterId, WorldBlueprintId};
use lib_soulfire::images::CharacterImage;
use lib_soulfire::strings::{
    CharacterDescription, CharacterName, CharacterPrompt, CharacterSubtitle, InitialMessageText,
    WorldDescription, WorldPrompt, WorldTitle,
};
use lib_soulfire::world::WorldBlueprint;

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

use super::Page;

const AVATAR_CHOICES: [CharacterImage; 8] = [
    CharacterImage::EmojiButterfly,
    CharacterImage::EmojiCat,
    CharacterImage::EmojiFox,
    CharacterImage::EmojiOwl,
    CharacterImage::EmojiDragon,
    CharacterImage::EmojiWolf,
    CharacterImage::EmojiPanda,
    CharacterImage::EmojiBee,
];

/// The character manual editor (`CHAR-1`..`CHAR-5`).
#[component]
pub fn CharacterEditor(id: Option<CharacterId>) -> Element {
    let app = current_app();
    let existing = id.as_ref().and_then(|i| app.store.character(i).ok().flatten());
    let is_edit = existing.is_some();

    let mut name = use_signal(|| existing.as_ref().map(|c| c.name.to_string()).unwrap_or_default());
    let mut subtitle = use_signal(|| existing.as_ref().map(|c| c.subtitle.to_string()).unwrap_or_default());
    let mut description = use_signal(|| existing.as_ref().map(|c| c.description.to_string()).unwrap_or_default());
    let mut prompt = use_signal(|| existing.as_ref().map(|c| c.prompt.to_string()).unwrap_or_default());
    let mut initial = use_signal(|| existing.as_ref().map(|c| c.initial_message.text().to_string()).unwrap_or_default());
    let mut is_prompt_initial = use_signal(|| existing.as_ref().map(|c| c.initial_message.is_prompt()).unwrap_or(false));
    let mut avatar = use_signal(|| existing.as_ref().and_then(|c| c.image).unwrap_or(CharacterImage::EmojiButterfly));
    let mut max_tokens = use_signal(|| existing.as_ref().map(|c| c.creativity.max_tokens).unwrap_or(2000));
    let mut temperature = use_signal(|| existing.as_ref().map(|c| c.creativity.temperature).unwrap_or(1.0));

    let save = use_callback(move |_: ()| {
        // Validate required fields (CHAR-5).
        if name().trim().is_empty() {
            ToastService::error("Name is required.");
            return;
        }
        if prompt().trim().is_empty() {
            ToastService::error("Prompt is required.");
            return;
        }
        if initial().trim().is_empty() {
            ToastService::error("Initial message is required.");
            return;
        }
        let text = InitialMessageText::coerce(initial().trim());
        let initial_message = if is_prompt_initial() {
            InitialMessage::Prompt(text)
        } else {
            InitialMessage::Message(text)
        };
        let mut character = existing.clone().unwrap_or_else(|| {
            Character::builder()
                .name(CharacterName::coerce("placeholder"))
                .initial_message(InitialMessage::default())
                .build()
        });
        character.name = CharacterName::coerce(name().trim());
        character.subtitle = CharacterSubtitle::coerce(subtitle().trim());
        character.description = CharacterDescription::coerce(description().trim());
        character.prompt = CharacterPrompt::coerce(prompt().trim());
        character.initial_message = initial_message;
        character.image = Some(avatar());
        character.creativity = CreativityControls {
            max_tokens: max_tokens(),
            temperature: temperature(),
            ..character.creativity
        }
        .clamped();
        character.updated_at = sp_core::datetime::SpDateTime::now();
        let _ = app.store.save_character(&character);
        data::touch();
        ToastService::info("Saved.");
        navigate(Screen::Characters);
    });

    rsx! {
        Page { title: if is_edit { "Edit Character".to_string() } else { "New Character".to_string() },
            // Avatar.
            Field { label: "Avatar".to_string(),
                div { class: "flex flex-wrap gap-2",
                    for choice in AVATAR_CHOICES {
                        button {
                            class: if avatar() == choice { "w-11 h-11 rounded-full bg-primary-lighter ring-2 ring-primary text-2xl flex items-center justify-center" } else { "w-11 h-11 rounded-full bg-surface border border-border text-2xl flex items-center justify-center" },
                            onclick: move |_| avatar.set(choice),
                            "{choice.emoji().unwrap_or(\"🙂\")}"
                        }
                    }
                }
            }
            Field { label: "Name".to_string(),
                input { class: "input-premium w-full", value: "{name}", oninput: move |e| name.set(e.value()) }
            }
            Field { label: "Subtitle".to_string(),
                input { class: "input-premium w-full", value: "{subtitle}", oninput: move |e| subtitle.set(e.value()) }
            }
            Field { label: "Description (not sent to AI)".to_string(),
                textarea { class: "input-premium w-full", rows: "2", value: "{description}", oninput: move |e| description.set(e.value()) }
            }
            Field { label: "Prompt — the personality the AI uses".to_string(),
                textarea { class: "input-premium w-full font-mono text-sm", rows: "8", value: "{prompt}", oninput: move |e| prompt.set(e.value()) }
            }
            Field { label: "Initial message".to_string(),
                div { class: "flex gap-2 mb-2",
                    TypeToggle { label: "Direct message".to_string(), active: !is_prompt_initial(), onclick: move |_| is_prompt_initial.set(false) }
                    TypeToggle { label: "Prompt (AI generates)".to_string(), active: is_prompt_initial(), onclick: move |_| is_prompt_initial.set(true) }
                }
                textarea { class: "input-premium w-full", rows: "3", value: "{initial}", oninput: move |e| initial.set(e.value()) }
            }
            Field { label: "Creativity".to_string(),
                div { class: "grid grid-cols-2 gap-3",
                    div {
                        p { class: "text-xs text-secondary-text mb-1", "Max tokens (500–5000)" }
                        input { class: "input-premium w-full", r#type: "number", value: "{max_tokens}",
                            oninput: move |e| if let Ok(v) = e.value().parse() { max_tokens.set(v) } }
                    }
                    div {
                        p { class: "text-xs text-secondary-text mb-1", "Temperature (0–2)" }
                        input { class: "input-premium w-full", r#type: "number", step: "0.1", value: "{temperature}",
                            oninput: move |e| if let Ok(v) = e.value().parse() { temperature.set(v) } }
                    }
                }
            }
            div { class: "flex gap-2 mt-4",
                button { class: "crm-primary-button px-6 py-2.5 rounded-lg", onclick: move |_| save(()), "Save" }
                button { class: "px-6 py-2.5 rounded-lg border border-border text-secondary-text", onclick: move |_| navigate(Screen::Characters), "Cancel" }
            }
        }
    }
}

/// The world blueprint manual editor (`WORLD-2`).
#[component]
pub fn WorldEditor(id: Option<WorldBlueprintId>) -> Element {
    let app = current_app();
    let existing = id.as_ref().and_then(|i| app.store.blueprint(i).ok().flatten());
    let is_edit = existing.is_some();

    let mut title = use_signal(|| existing.as_ref().map(|b| b.title.to_string()).unwrap_or_default());
    let mut description = use_signal(|| existing.as_ref().map(|b| b.description.to_string()).unwrap_or_default());
    let mut world_prompt = use_signal(|| existing.as_ref().map(|b| b.world_prompt.to_string()).unwrap_or_default());

    let save = use_callback(move |_: ()| {
        if title().trim().is_empty() {
            ToastService::error("Title is required.");
            return;
        }
        if world_prompt().trim().is_empty() {
            ToastService::error("World prompt is required.");
            return;
        }
        let mut bp = existing.clone().unwrap_or_else(|| {
            WorldBlueprint::builder()
                .title(WorldTitle::coerce("placeholder"))
                .world_prompt(WorldPrompt::coerce("placeholder"))
                .build()
        });
        bp.title = WorldTitle::coerce(title().trim());
        bp.description = WorldDescription::coerce(description().trim());
        bp.world_prompt = WorldPrompt::coerce(world_prompt().trim());
        bp.updated_at = sp_core::datetime::SpDateTime::now();
        let _ = app.store.save_blueprint(&bp);
        data::touch();
        ToastService::info("Saved.");
        navigate(Screen::WorldsHome);
    });

    rsx! {
        Page { title: if is_edit { "Edit World".to_string() } else { "New World".to_string() },
            Field { label: "Title".to_string(),
                input { class: "input-premium w-full", value: "{title}", oninput: move |e| title.set(e.value()) }
            }
            Field { label: "Description (shown to player, not sent to AI)".to_string(),
                textarea { class: "input-premium w-full", rows: "2", value: "{description}", oninput: move |e| description.set(e.value()) }
            }
            Field { label: "World prompt — premise, lore, NPCs, rules, acts".to_string(),
                textarea { class: "input-premium w-full font-mono text-sm", rows: "16", value: "{world_prompt}", oninput: move |e| world_prompt.set(e.value()) }
            }
            div { class: "flex gap-2 mt-4",
                button { class: "crm-primary-button px-6 py-2.5 rounded-lg", onclick: move |_| save(()), "Save" }
                button { class: "px-6 py-2.5 rounded-lg border border-border text-secondary-text", onclick: move |_| navigate(Screen::WorldsHome), "Cancel" }
            }
        }
    }
}

#[component]
fn Field(label: String, children: Element) -> Element {
    rsx! {
        div { class: "mb-4",
            label { class: "block text-sm font-semibold text-primary-light mb-1.5", "{label}" }
            {children}
        }
    }
}

#[component]
fn TypeToggle(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active { "bg-primary text-primary-text" } else { "bg-surface border border-border text-secondary-text" };
    rsx! {
        button { class: "px-3 py-1.5 rounded-lg text-sm {cls}", onclick: move |e| onclick.call(e), "{label}" }
    }
}

// Keep FromStr in scope for potential strict parsing of bounded fields.
#[allow(unused)]
fn _use_from_str() {
    let _ = CharacterName::from_str("x");
}
