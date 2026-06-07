//! Worlds home: Adventures (continue) and Worlds (start/create) tabs (`UI-8`,
//! `UI-9`, `ONB-6`).

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::world::{Adventure, WorldBlueprint};

use soulfire_core::store::ImageOwnerKind;

use crate::app::current_app;
use crate::data;
use crate::image_util::data_uri;
use crate::nav::{Screen, navigate};

use super::{EmptyState, Page};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Adventures,
    Worlds,
}

#[component]
pub fn WorldsHome() -> Element {
    data::subscribe();
    let app = current_app();
    let mut tab = use_signal(|| Tab::Adventures);

    let mut search = use_signal(String::new);
    let q = search();
    let query = if q.trim().is_empty() { None } else { Some(q.trim()) };
    let adventures = app.store.list_adventures(50, 0).unwrap_or_default();
    let worlds = app.store.list_blueprints(query, 100, 0).unwrap_or_default();

    rsx! {
        Page { title: "Worlds".to_string(),
            div { class: "flex gap-2 mb-5 border-b border-border",
                TabButton { active: tab() == Tab::Adventures, label: "Adventures".to_string(), onclick: move |_| tab.set(Tab::Adventures) }
                TabButton { active: tab() == Tab::Worlds, label: "Worlds".to_string(), onclick: move |_| tab.set(Tab::Worlds) }
            }

            match tab() {
                Tab::Adventures => rsx! {
                    if adventures.is_empty() {
                        EmptyState { message: "No adventures yet. Open a world to begin.".to_string() }
                    } else {
                        div { class: "grid gap-4 sm:grid-cols-2",
                            for adv in adventures.clone() { AdventureCard { adventure: adv } }
                        }
                    }
                },
                Tab::Worlds => rsx! {
                    div { class: "flex justify-end gap-2 mb-3",
                        button {
                            class: "px-4 py-2 rounded-lg text-sm border border-border text-primary-text hover-highlight",
                            onclick: move |_| {
                                use lib_soulfire::strings::{WorldPrompt, WorldTitle};
                                let b = WorldBlueprint::builder()
                                    .title(WorldTitle::coerce("New World"))
                                    .world_prompt(WorldPrompt::coerce("A new world to shape."))
                                    .build();
                                let _ = app.store.save_blueprint(&b);
                                data::touch();
                                navigate(Screen::WorldBuilder(b.blueprint_id));
                            },
                            "✨ World Builder"
                        }
                        button {
                            class: "crm-primary-button px-4 py-2 rounded-lg text-sm",
                            onclick: move |_| navigate(Screen::WorldEditor(None)),
                            "+ New World"
                        }
                    }
                    input {
                        class: "input-premium w-full mb-3",
                        placeholder: "Search worlds…",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                    if worlds.is_empty() {
                        EmptyState { message: "No worlds yet. Create one to start an adventure.".to_string() }
                    } else {
                        div { class: "grid gap-4 sm:grid-cols-2",
                            for bp in worlds.clone() { WorldCard { blueprint: bp } }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn TabButton(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active {
        "border-primary text-primary"
    } else {
        "border-transparent text-secondary-text"
    };
    rsx! {
        button {
            class: "px-4 py-2 -mb-px border-b-2 font-medium {cls}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// 16:6 cover: a stored image when present, else an emoji over an accent gradient
/// (`UI-9`, `IMG-8`).
#[component]
fn Cover(emoji: String, completed: bool, #[props(default)] uri: Option<String>) -> Element {
    rsx! {
        div { class: "relative w-full rounded-lg overflow-hidden mb-3",
            style: "aspect-ratio: 16 / 6; background: linear-gradient(135deg, var(--color-primary-darker), var(--color-primary-darkest));",
            if let Some(uri) = uri {
                img { class: "absolute inset-0 w-full h-full object-cover", src: "{uri}" }
            } else {
                div { class: "absolute inset-0 flex items-center justify-center text-5xl opacity-90", "{emoji}" }
            }
            if completed {
                span { class: "absolute top-2 right-2 text-xs bg-black/40 text-white px-2 py-0.5 rounded", "Completed" }
            }
        }
    }
}

#[component]
fn AdventureCard(adventure: Adventure) -> Element {
    let app = current_app();
    let title = adventure
        .world_title
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Adventure".to_string());
    let emoji = adventure.world_image.map(|i| i.emoji()).unwrap_or("🌍").to_string();
    let desc = adventure
        .world_description
        .as_ref()
        .map(|d| d.to_string())
        .unwrap_or_default();
    let adv_id = adventure.adventure_id.clone();
    let del_id = adventure.adventure_id.clone();
    let uri = adventure
        .world_cover
        .and_then(|_| data_uri(&app, ImageOwnerKind::World, &adventure.blueprint_id.to_string()));
    let mut confirming = use_signal(|| false);
    rsx! {
        div { class: "bg-surface border border-border rounded-xl p-4",
            Cover { emoji, completed: adventure.has_completed, uri }
            h3 { class: "font-semibold text-primary-text mb-1", "{title}" }
            p { class: "text-sm text-secondary-text line-clamp-2 mb-3", "{desc}" }
            div { class: "flex gap-2",
                button {
                    class: "crm-primary-button flex-1 py-2 rounded-lg text-sm",
                    onclick: move |_| navigate(Screen::Play(adv_id.clone())),
                    if adventure.has_completed { "Review Adventure" } else { "Continue Adventure" }
                }
                button {
                    class: "px-3 py-2 rounded-lg border border-border text-secondary-text hover-highlight text-sm",
                    onclick: move |_| confirming.set(true),
                    "Delete"
                }
            }
            crate::components::ConfirmDialog {
                open: confirming(),
                title: "Delete adventure?".to_string(),
                message: "This permanently deletes this playthrough and its turn log.".to_string(),
                danger: true,
                confirm_label: "Delete".to_string(),
                on_confirm: move |_| {
                    let _ = app.store.delete_adventure(&del_id);
                    confirming.set(false);
                    data::touch();
                },
                on_cancel: move |_| confirming.set(false),
            }
        }
    }
}

#[component]
fn WorldCard(blueprint: WorldBlueprint) -> Element {
    let app = current_app();
    let emoji = blueprint.image.map(|i| i.emoji()).unwrap_or("🌍").to_string();
    let uri = blueprint
        .cover
        .and_then(|_| data_uri(&app, ImageOwnerKind::World, &blueprint.blueprint_id.to_string()));
    let app_del = app.clone();
    let bp = blueprint.clone();
    let bp_edit = blueprint.blueprint_id.clone();
    let bp_del = blueprint.blueprint_id.clone();
    let mut starting = use_signal(|| false);
    let mut confirming = use_signal(|| false);

    rsx! {
        div { class: "bg-surface border border-border rounded-xl p-4",
            Cover { emoji, completed: false, uri }
            h3 { class: "font-semibold text-primary-text mb-1", "{blueprint.title}" }
            p { class: "text-sm text-secondary-text line-clamp-2 mb-3", "{blueprint.description}" }
            crate::components::ConfirmDialog {
                open: confirming(),
                title: "Delete world?".to_string(),
                message: "This permanently deletes the world and all of its adventures.".to_string(),
                danger: true,
                confirm_label: "Delete".to_string(),
                confirm_word: Some("delete".to_string()),
                on_confirm: move |_| {
                    let _ = app_del.store.delete_blueprint(&bp_del);
                    confirming.set(false);
                    data::touch();
                },
                on_cancel: move |_| confirming.set(false),
            }
            div { class: "flex gap-2",
                button {
                    class: "crm-primary-button flex-1 py-2 rounded-lg text-sm disabled:opacity-50",
                    disabled: starting(),
                    onclick: move |_| {
                        let app = app.clone();
                        let bp = bp.clone();
                        starting.set(true);
                        spawn(async move {
                            match app.world.start_adventure(&bp, |_| {}).await {
                                Ok(adv) => { data::touch(); navigate(Screen::Play(adv.adventure_id)); }
                                Err(e) => ToastService::error(&format!("Could not start: {e}")),
                            }
                            starting.set(false);
                        });
                    },
                    if starting() { "Starting…" } else { "Enter World" }
                }
                button {
                    class: "px-3 py-2 rounded-lg border border-border text-secondary-text hover-highlight text-sm",
                    onclick: move |_| navigate(Screen::WorldEditor(Some(bp_edit.clone()))),
                    "Edit"
                }
                button {
                    class: "px-3 py-2 rounded-lg border border-border text-secondary-text hover-highlight text-sm",
                    onclick: move |_| confirming.set(true),
                    "🗑"
                }
            }
        }
    }
}
