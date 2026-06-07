//! Manual editors for characters and world blueprints (`UI-18`, `CHAR-1`..`CHAR-5`,
//! `WORLD-2`).

use std::str::FromStr;

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::character::{Character, CreativityControls, InitialMessage};
use lib_soulfire::ids::{CharacterId, WorldBlueprintId};
use lib_soulfire::images::{CharacterImage, ImageTransform};
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
            // Portrait (generated/uploaded) with crop, or emoji avatar (IMG-7/8).
            Field { label: "Portrait".to_string(), PortraitSection { character_id: id.clone() } }
            // Avatar emoji.
            Field { label: "Avatar (used when no portrait)".to_string(),
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
            div { class: "flex gap-2 mt-4 items-center",
                button { class: "crm-primary-button px-6 py-2.5 rounded-lg", onclick: move |_| save(()), "Save" }
                button { class: "px-6 py-2.5 rounded-lg border border-border text-secondary-text", onclick: move |_| navigate(Screen::Characters), "Cancel" }
                if let Some(cid) = id.clone() {
                    {
                        let cid_b = cid.clone();
                        rsx! {
                            button {
                                class: "ml-auto text-link text-sm hover:underline",
                                onclick: move |_| navigate(Screen::CharacterBuilder(cid_b.clone())),
                                "Builder →"
                            }
                            button {
                                class: "text-link text-sm hover:underline",
                                onclick: move |_| navigate(Screen::PromptViewer(cid.clone())),
                                "System prompt →"
                            }
                        }
                    }
                }
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
            Field { label: "Cover".to_string(), CoverSection { blueprint_id: id.clone() } }
            Field { label: "Title".to_string(),
                input { class: "input-premium w-full", value: "{title}", oninput: move |e| title.set(e.value()) }
            }
            Field { label: "Description (shown to player, not sent to AI)".to_string(),
                textarea { class: "input-premium w-full", rows: "2", value: "{description}", oninput: move |e| description.set(e.value()) }
            }
            Field { label: "World prompt — premise, lore, NPCs, rules, acts".to_string(),
                textarea { class: "input-premium w-full font-mono text-sm", rows: "16", value: "{world_prompt}", oninput: move |e| world_prompt.set(e.value()) }
            }
            div { class: "flex gap-2 mt-4 items-center",
                button { class: "crm-primary-button px-6 py-2.5 rounded-lg", onclick: move |_| save(()), "Save" }
                button { class: "px-6 py-2.5 rounded-lg border border-border text-secondary-text", onclick: move |_| navigate(Screen::WorldsHome), "Cancel" }
                if let Some(bid) = id.clone() {
                    button {
                        class: "ml-auto text-link text-sm hover:underline",
                        onclick: move |_| navigate(Screen::WorldBuilder(bid.clone())),
                        "Builder →"
                    }
                }
            }
        }
    }
}

/// Character portrait: generate/regenerate/clear and a pan/zoom/reset framing
/// editor (`IMG-1`..`IMG-3`, `IMG-7`, `IMG-8`).
#[component]
fn PortraitSection(character_id: Option<CharacterId>) -> Element {
    use soulfire_core::store::ImageOwnerKind;
    let app = current_app();
    let Some(cid) = character_id else {
        return rsx! { p { class: "text-sm text-secondary-text", "Save the character first to add a portrait." } };
    };
    let character = app.store.character(&cid).ok().flatten();
    let has = character.as_ref().and_then(|c| c.portrait).is_some();
    let transform = character.as_ref().map(|c| c.image_transform).unwrap_or_default();
    let uri = if has {
        crate::image_util::data_uri(&app, ImageOwnerKind::Character, &cid.to_string())
    } else {
        None
    };
    let mut generating = use_signal(|| false);

    let regen = {
        let app = app.clone();
        let cid = cid.clone();
        use_callback(move |_: ()| {
            if !app.has_api_key() {
                ToastService::info("Add your OpenAI key in Settings to generate images.");
                return;
            }
            let app = app.clone();
            let cid = cid.clone();
            generating.set(true);
            spawn(async move {
                match app.image.generate_character_portrait(&cid).await {
                    Ok(_) => data::touch(),
                    Err(e) => ToastService::error(&format!("{e}")),
                }
                generating.set(false);
            });
        })
    };
    let clear = {
        let app = app.clone();
        let cid = cid.clone();
        use_callback(move |_: ()| {
            let _ = app.image.clear_character_portrait(&cid);
            data::touch();
        })
    };
    let set_transform = {
        let app = app.clone();
        let cid = cid.clone();
        use_callback(move |t: ImageTransform| {
            if let Some(mut c) = app.store.character(&cid).ok().flatten() {
                c.image_transform = t;
                let _ = app.store.save_character(&c);
                data::touch();
            }
        })
    };

    rsx! {
        if let Some(uri) = uri {
            div { class: "flex flex-col sm:flex-row items-start gap-4",
                FrameEditor { uri, transform, round: true, max_zoom: 240, on_change: move |t| set_transform(t) }
                div { class: "flex gap-2",
                    button { class: "px-3 py-1.5 rounded-lg border border-border text-sm text-secondary-text hover-highlight disabled:opacity-40", disabled: generating(), onclick: move |_| regen(()), if generating() { "…" } else { "Regenerate" } }
                    button { class: "px-3 py-1.5 rounded-lg border border-border text-sm text-secondary-text hover-highlight", onclick: move |_| clear(()), "Remove" }
                }
            }
        } else {
            button {
                class: "px-4 py-2 rounded-lg border border-border text-primary-text hover-highlight disabled:opacity-40",
                disabled: generating(),
                onclick: move |_| regen(()),
                if generating() { "Generating…" } else { "✨ Generate portrait" }
            }
        }
    }
}

/// World cover: generate/clear and framing (`IMG`).
#[component]
fn CoverSection(blueprint_id: Option<WorldBlueprintId>) -> Element {
    use soulfire_core::store::ImageOwnerKind;
    let app = current_app();
    let Some(bid) = blueprint_id else {
        return rsx! { p { class: "text-sm text-secondary-text", "Save the world first to add a cover." } };
    };
    let bp = app.store.blueprint(&bid).ok().flatten();
    let has = bp.as_ref().and_then(|b| b.cover).is_some();
    let transform = bp.as_ref().map(|b| b.image_transform).unwrap_or_default();
    let uri = if has {
        crate::image_util::data_uri(&app, ImageOwnerKind::World, &bid.to_string())
    } else {
        None
    };
    let mut generating = use_signal(|| false);
    let regen = {
        let app = app.clone();
        let bid = bid.clone();
        use_callback(move |_: ()| {
            if !app.has_api_key() {
                ToastService::info("Add your OpenAI key in Settings to generate images.");
                return;
            }
            let app = app.clone();
            let bid = bid.clone();
            generating.set(true);
            spawn(async move {
                match app.image.generate_world_cover(&bid).await {
                    Ok(_) => data::touch(),
                    Err(e) => ToastService::error(&format!("{e}")),
                }
                generating.set(false);
            });
        })
    };
    let clear = {
        let app = app.clone();
        let bid = bid.clone();
        use_callback(move |_: ()| {
            let _ = app.image.clear_world_cover(&bid);
            data::touch();
        })
    };

    let set_transform = {
        let app = app.clone();
        let bid = bid.clone();
        use_callback(move |t: ImageTransform| {
            if let Some(mut b) = app.store.blueprint(&bid).ok().flatten() {
                b.image_transform = t;
                let _ = app.store.save_blueprint(&b);
                data::touch();
            }
        })
    };
    rsx! {
        if let Some(uri) = uri {
            div {
                FrameEditor { uri, transform, round: false, max_zoom: 220, on_change: move |t| set_transform(t) }
                div { class: "flex gap-2 mt-2",
                    button { class: "px-3 py-1.5 rounded-lg border border-border text-sm text-secondary-text hover-highlight disabled:opacity-40", disabled: generating(), onclick: move |_| regen(()), if generating() { "…" } else { "Regenerate" } }
                    button { class: "px-3 py-1.5 rounded-lg border border-border text-sm text-secondary-text hover-highlight", onclick: move |_| clear(()), "Remove" }
                }
            }
        } else {
            button {
                class: "px-4 py-2 rounded-lg border border-border text-primary-text hover-highlight disabled:opacity-40",
                disabled: generating(),
                onclick: move |_| regen(()),
                if generating() { "Generating…" } else { "✨ Generate cover" }
            }
        }
    }
}

/// A framing editor: drag-to-pan over the image, a zoom slider, and reset, on
/// pointer/touch input (`IMG-7`). Pan updates a local working transform during a
/// drag and persists it on release (one store write per gesture).
#[component]
fn FrameEditor(
    uri: String,
    transform: ImageTransform,
    #[props(default)] round: bool,
    max_zoom: u16,
    on_change: EventHandler<ImageTransform>,
) -> Element {
    let mut working = use_signal(|| transform);
    let mut drag = use_signal(|| None::<(f64, f64)>);
    // Keep the working copy in sync with the persisted value when not dragging.
    use_effect(move || {
        if drag().is_none() {
            working.set(transform);
        }
    });

    let frame_cls = if round {
        "w-40 h-40 rounded-full"
    } else {
        "w-full rounded-lg"
    };
    let frame_style = if round { "" } else { "aspect-ratio: 16 / 6;" };

    rsx! {
        div { class: "w-full",
            div {
                class: "overflow-hidden bg-surface border border-border cursor-move select-none touch-none {frame_cls}",
                style: "{frame_style}",
                onpointerdown: move |e| {
                    let p = e.client_coordinates();
                    drag.set(Some((p.x, p.y)));
                },
                onpointermove: move |e| {
                    if let Some((lx, ly)) = drag() {
                        let p = e.client_coordinates();
                        let cur = working();
                        working.set(ImageTransform {
                            pan_x_percent: (cur.pan_x_percent as f64 + (p.x - lx) * 0.3).clamp(-100.0, 100.0) as i16,
                            pan_y_percent: (cur.pan_y_percent as f64 + (p.y - ly) * 0.3).clamp(-100.0, 100.0) as i16,
                            ..cur
                        });
                        drag.set(Some((p.x, p.y)));
                    }
                },
                onpointerup: move |_| if drag().take().is_some() { drag.set(None); on_change.call(working()); },
                onpointerleave: move |_| if drag().is_some() { drag.set(None); on_change.call(working()); },
                img { class: "pointer-events-none", src: "{uri}", style: crate::image_util::transform_style(working()) }
            }
            div { class: "mt-2",
                label { class: "text-xs text-secondary-text", "Zoom" }
                input {
                    class: "w-full", r#type: "range", min: "100", max: "{max_zoom}", value: "{working().zoom_percent}",
                    oninput: move |e| if let Ok(v) = e.value().parse::<u16>() {
                        let t = ImageTransform { zoom_percent: v, ..working() };
                        working.set(t);
                        on_change.call(t);
                    },
                }
                button {
                    class: "text-xs text-link hover:underline",
                    onclick: move |_| { working.set(ImageTransform::default()); on_change.call(ImageTransform::default()); },
                    "Reset framing"
                }
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
