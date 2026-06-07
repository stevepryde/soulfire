//! Worlds home: Adventures (continue) and Worlds (start/create) tabs (`UI-8`,
//! `UI-9`, `ONB-6`). Card/tab/cover markup ported from Soulfire-OG's
//! `pages/worlds/home.rs` + `components/world.rs`.

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::ids::{AdventureId, WorldBlueprintId};
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

/// Worlds/adventures page size (keyset paging, `UI-22`).
const PAGE: u32 = 20;

#[component]
pub fn WorldsHome() -> Element {
    let app = current_app();
    let mut tab = use_signal(|| Tab::Adventures);
    let mut search = use_signal(String::new);

    // Accumulated keyset pages; each "Load more" appends the next page only.
    let mut adventures = use_signal(Vec::<Adventure>::new);
    let mut adv_more = use_signal(|| false);
    let mut worlds = use_signal(Vec::<WorldBlueprint>::new);
    let mut world_more = use_signal(|| false);

    // Adventures (unfiltered): reload page 1 on any store mutation.
    {
        let app = app.clone();
        use_effect(move || {
            data::subscribe();
            let page = app.store.list_adventures(None, PAGE).unwrap_or_default();
            adv_more.set(page.len() as u32 == PAGE);
            adventures.set(page);
        });
    }
    let load_more_adv = {
        let app = app.clone();
        use_callback(move |_: ()| {
            let cursor = adventures.read().last().cloned();
            let page = app
                .store
                .list_adventures(cursor.as_ref(), PAGE)
                .unwrap_or_default();
            adv_more.set(page.len() as u32 == PAGE);
            adventures.write().extend(page);
        })
    };

    // Worlds: reload page 1 on search text or store changes.
    {
        let app = app.clone();
        use_effect(move || {
            data::subscribe();
            let q = search();
            let query = if q.trim().is_empty() {
                None
            } else {
                Some(q.trim())
            };
            let page = app
                .store
                .list_blueprints(query, None, PAGE)
                .unwrap_or_default();
            world_more.set(page.len() as u32 == PAGE);
            worlds.set(page);
        });
    }
    let load_more_world = {
        let app = app.clone();
        use_callback(move |_: ()| {
            let q = search();
            let query = if q.trim().is_empty() {
                None
            } else {
                Some(q.trim())
            };
            let cursor = worlds.read().last().cloned();
            let page = app
                .store
                .list_blueprints(query, cursor.as_ref(), PAGE)
                .unwrap_or_default();
            world_more.set(page.len() as u32 == PAGE);
            worlds.write().extend(page);
        })
    };

    let adv_list = adventures();
    let world_list = worlds();

    rsx! {
        Page { title: "Worlds".to_string(),
            // Tab bar (OG `WorldsNavButton`).
            div { class: "flex flex-wrap items-center gap-2 mb-5",
                TabButton { active: tab() == Tab::Adventures, label: "Adventures".to_string(), onclick: move |_| tab.set(Tab::Adventures) }
                TabButton { active: tab() == Tab::Worlds, label: "Worlds".to_string(), onclick: move |_| tab.set(Tab::Worlds) }
            }

            match tab() {
                Tab::Adventures => rsx! {
                    if adv_list.is_empty() {
                        EmptyState { message: "No adventures yet. Open a world to begin.".to_string() }
                    } else {
                        div { class: "grid grid-cols-1 gap-4 lg:grid-cols-2",
                            for adv in adv_list.clone() {
                                AdventureCard {
                                    adventure: adv,
                                    on_delete: move |id: AdventureId| {
                                        adventures.write().retain(|a| a.adventure_id != id);
                                    },
                                }
                            }
                        }
                        if adv_more() {
                            div { class: "flex justify-center pt-4",
                                button {
                                    class: "rounded-full border border-white/8 bg-white/[0.04] px-4 py-2 text-sm font-medium text-white/68 transition-colors hover:bg-white/[0.08] hover:text-white cursor-pointer",
                                    onclick: move |_| load_more_adv(()),
                                    "Load more"
                                }
                            }
                        }
                    }
                },
                Tab::Worlds => rsx! {
                    div { class: "flex flex-wrap items-center gap-2 mb-4",
                        button {
                            class: "inline-flex cursor-pointer items-center justify-center gap-2 rounded-[12px] border border-white/10 bg-white/[0.05] px-4 py-2.5 text-sm font-semibold text-white/78 transition-colors hover:bg-white/[0.09] hover:text-white",
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
                            lucide_dioxus::Sparkles { size: 16 }
                            "World Builder"
                        }
                        button {
                            class: "inline-flex cursor-pointer items-center justify-center gap-2 rounded-[12px] bg-primary px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-primary-dark active:bg-primary-darker",
                            onclick: move |_| navigate(Screen::WorldEditor(None)),
                            lucide_dioxus::Plus { size: 16 }
                            "New World"
                        }
                    }
                    // Search (OG search field).
                    div { class: "relative overflow-hidden rounded-[14px] border border-white/8 bg-[#1a1a1d] mb-4",
                        div { class: "pointer-events-none absolute left-4 top-1/2 -translate-y-1/2 text-white/35",
                            lucide_dioxus::Search { size: 18 }
                        }
                        input {
                            r#type: "text",
                            class: "w-full bg-transparent py-3.5 pl-12 pr-11 text-[15px] text-white placeholder:text-white/35 focus:outline-none",
                            placeholder: "Search worlds…",
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                        if !search().is_empty() {
                            button {
                                class: "absolute right-2.5 top-1/2 flex h-8 w-8 -translate-y-1/2 cursor-pointer items-center justify-center rounded-full text-white/40 transition-colors hover:bg-white/10 hover:text-white/80",
                                onclick: move |_| search.set(String::new()),
                                lucide_dioxus::X { size: 16 }
                            }
                        }
                    }
                    if world_list.is_empty() {
                        EmptyState { message: "No worlds yet. Create one to start an adventure.".to_string() }
                    } else {
                        div { class: "grid grid-cols-1 gap-5 lg:grid-cols-2 2xl:grid-cols-3",
                            for bp in world_list.clone() {
                                WorldCard {
                                    blueprint: bp,
                                    // Deleting a world cascades to its adventures, so drop
                                    // those from the adventures list too (no refetch).
                                    on_delete: move |id: WorldBlueprintId| {
                                        worlds.write().retain(|b| b.blueprint_id != id);
                                        adventures.write().retain(|a| a.blueprint_id != id);
                                    },
                                }
                            }
                        }
                        if world_more() {
                            div { class: "flex justify-center pt-4",
                                button {
                                    class: "rounded-full border border-white/8 bg-white/[0.04] px-4 py-2 text-sm font-medium text-white/68 transition-colors hover:bg-white/[0.08] hover:text-white cursor-pointer",
                                    onclick: move |_| load_more_world(()),
                                    "Load more"
                                }
                            }
                        }
                    }
                },
            }
        }
    }
}

/// A worlds-home tab pill (OG `WorldsNavButton`).
#[component]
fn TabButton(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active {
        "border-white/12 bg-white/[0.08] text-white"
    } else {
        "border-white/8 bg-[#161618] text-white/65 hover:bg-white/[0.05] hover:text-white"
    };
    rsx! {
        button {
            class: "inline-flex cursor-pointer items-center justify-center rounded-[12px] border px-4 py-2.5 text-sm font-medium transition-colors {cls}",
            onclick: move |e| onclick.call(e),
            "{label}"
        }
    }
}

/// 16:6 cover art: a stored image (background-cover) or an emoji fallback
/// (OG `WorldCoverMedia`).
#[component]
fn CoverArt(emoji: String, #[props(default)] uri: Option<String>) -> Element {
    rsx! {
        div { class: "relative isolate overflow-hidden rounded-[18px] border border-white/8 bg-[#101013] aspect-[16/6] w-full",
            if let Some(uri) = uri {
                div {
                    class: "pointer-events-none absolute inset-0 select-none",
                    style: "background-image: url('{uri}'); background-position: center; background-repeat: no-repeat; background-size: cover;",
                    role: "img",
                }
            } else {
                div { class: "absolute inset-0 flex items-center justify-center",
                    span { class: "select-none text-[4.3rem] drop-shadow-[0_12px_22px_rgba(0,0,0,0.34)] sm:text-[4.8rem]", "{emoji}" }
                }
            }
        }
    }
}

/// A story card: cover, title, description, metadata, and a primary action
/// (OG `SharedStoryCard`).
#[component]
fn StoryCard(
    emoji: String,
    #[props(default)] uri: Option<String>,
    title: String,
    description: String,
    action: Element,
    #[props(default)] top_right: Option<Element>,
    #[props(default)] metadata: Option<Element>,
) -> Element {
    let has_description = !description.trim().is_empty();
    rsx! {
        div { class: "relative isolate z-0 overflow-visible rounded-[24px] border border-white/8 bg-[#1b1b1e] p-3 shadow-[0_16px_36px_rgba(0,0,0,0.24)] sm:p-4 flex h-full flex-col",
            CoverArt { emoji, uri }
            div { class: "mt-4 flex flex-1 flex-col",
                div { class: "flex items-start justify-between gap-3",
                    div { class: "min-w-0",
                        h3 { class: "truncate text-[1.7rem] font-semibold tracking-tight text-white", "{title}" }
                    }
                    if let Some(top_right) = top_right {
                        {top_right}
                    }
                }
                if has_description {
                    p { class: "mt-2 line-clamp-3 text-sm leading-6 text-white/62", "{description}" }
                }
                if let Some(metadata) = metadata {
                    div { class: "mt-3", {metadata} }
                }
                div { class: "mt-auto pt-4", {action} }
            }
        }
    }
}

/// The "Completed" status pill (OG `CompletedAdventurePill`).
#[component]
fn CompletedPill() -> Element {
    rsx! {
        div { class: "flex flex-wrap items-center gap-2",
            span {
                class: "inline-flex items-center gap-1.5 rounded-full border border-emerald-400/30 bg-emerald-400/12 px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.12em] text-emerald-100",
                lucide_dioxus::CircleCheck { size: 13 }
                "Completed"
            }
        }
    }
}

#[component]
fn AdventureCard(adventure: Adventure, on_delete: EventHandler<AdventureId>) -> Element {
    let app = current_app();
    let title = adventure
        .world_title
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Adventure".to_string());
    let emoji = adventure
        .world_image
        .map(|i| i.emoji())
        .unwrap_or("🌍")
        .to_string();
    let desc = adventure
        .world_description
        .as_ref()
        .map(|d| d.to_string())
        .unwrap_or_default();
    let completed = adventure.has_completed;
    let adv_id = adventure.adventure_id.clone();
    let del_id = adventure.adventure_id.clone();
    let uri = adventure.world_cover.and_then(|_| {
        data_uri(
            &app,
            ImageOwnerKind::World,
            &adventure.blueprint_id.to_string(),
        )
    });
    let mut confirming = use_signal(|| false);

    let action = rsx! {
        button {
            class: "inline-flex w-full cursor-pointer items-center justify-center gap-2 rounded-[12px] bg-primary px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-primary-dark active:bg-primary-darker",
            onclick: move |_| navigate(Screen::Play(adv_id.clone())),
            lucide_dioxus::Play { size: 16 }
            if completed { "Review Adventure" } else { "Continue Adventure" }
        }
    };
    let top_right = rsx! {
        button {
            class: "cursor-pointer rounded-full p-2 text-white/38 transition-colors hover:bg-white/8 hover:text-white/80",
            onclick: move |_| confirming.set(true),
            lucide_dioxus::Trash2 { size: 18 }
        }
    };
    let metadata = completed.then(|| rsx! { CompletedPill {} });

    rsx! {
        StoryCard { emoji, uri, title, description: desc, action, top_right: Some(top_right), metadata }
        crate::components::ConfirmDialog {
            open: confirming(),
            title: "Delete adventure?".to_string(),
            message: "This permanently deletes this playthrough and its turn log.".to_string(),
            danger: true,
            confirm_label: "Delete".to_string(),
            on_confirm: move |_| {
                if app.store.delete_adventure(&del_id).is_ok() {
                    on_delete.call(del_id.clone());
                }
                confirming.set(false);
            },
            on_cancel: move |_| confirming.set(false),
        }
    }
}

#[component]
fn WorldCard(blueprint: WorldBlueprint, on_delete: EventHandler<WorldBlueprintId>) -> Element {
    let app = current_app();
    let emoji = blueprint
        .image
        .map(|i| i.emoji())
        .unwrap_or("🌍")
        .to_string();
    let title = blueprint.title.to_string();
    let desc = blueprint.description.to_string();
    let uri = blueprint.cover.and_then(|_| {
        data_uri(
            &app,
            ImageOwnerKind::World,
            &blueprint.blueprint_id.to_string(),
        )
    });
    let app_del = app.clone();
    let bp = blueprint.clone();
    let bp_edit = blueprint.blueprint_id.clone();
    let bp_build = blueprint.blueprint_id.clone();
    let bp_del = blueprint.blueprint_id.clone();
    let mut starting = use_signal(|| false);
    let mut confirming = use_signal(|| false);

    let action = rsx! {
        div { class: "flex flex-col gap-2",
            button {
                class: "inline-flex w-full cursor-pointer items-center justify-center gap-2 rounded-[12px] bg-primary px-4 py-3 text-sm font-semibold text-white transition-colors hover:bg-primary-dark active:bg-primary-darker disabled:opacity-50",
                disabled: starting(),
                onclick: move |_| {
                    let app = app.clone();
                    let bp = bp.clone();
                    starting.set(true);
                    spawn(async move {
                        match app.world.start_adventure(&bp, |_| {}).await {
                            Ok(adv) => {
                                data::touch();
                                navigate(Screen::Play(adv.adventure_id));
                            }
                            Err(e) => ToastService::error(&format!("Could not start: {e}")),
                        }
                        starting.set(false);
                    });
                },
                lucide_dioxus::DoorOpen { size: 16 }
                if starting() { "Starting…" } else { "Enter World" }
            }
            div { class: "flex gap-2",
                button {
                    class: "inline-flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-[12px] border border-white/10 bg-white/[0.05] px-4 py-2.5 text-sm font-semibold text-white/78 transition-colors hover:bg-white/[0.09] hover:text-white",
                    onclick: move |_| navigate(Screen::WorldBuilder(bp_build.clone())),
                    lucide_dioxus::Sparkles { size: 16 }
                    "Build"
                }
                button {
                    class: "inline-flex flex-1 cursor-pointer items-center justify-center gap-2 rounded-[12px] border border-white/10 bg-white/[0.05] px-4 py-2.5 text-sm font-semibold text-white/78 transition-colors hover:bg-white/[0.09] hover:text-white",
                    onclick: move |_| navigate(Screen::WorldEditor(Some(bp_edit.clone()))),
                    lucide_dioxus::Pencil { size: 16 }
                    "Edit"
                }
            }
        }
    };
    let top_right = rsx! {
        button {
            class: "cursor-pointer rounded-full p-2 text-white/38 transition-colors hover:bg-white/8 hover:text-white/80",
            onclick: move |_| confirming.set(true),
            lucide_dioxus::Trash2 { size: 18 }
        }
    };

    rsx! {
        StoryCard { emoji, uri, title, description: desc, action, top_right: Some(top_right) }
        crate::components::ConfirmDialog {
            open: confirming(),
            title: "Delete world?".to_string(),
            message: "This permanently deletes the world and all of its adventures.".to_string(),
            danger: true,
            confirm_label: "Delete".to_string(),
            confirm_word: Some("delete".to_string()),
            on_confirm: move |_| {
                if app_del.store.delete_blueprint(&bp_del).is_ok() {
                    on_delete.call(bp_del.clone());
                }
                confirming.set(false);
            },
            on_cancel: move |_| confirming.set(false),
        }
    }
}
