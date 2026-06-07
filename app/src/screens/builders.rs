//! The conversational character and world builders (`CHAR-6`..`CHAR-9`,
//! `WORLD-20`, `WORLD-21`). A Chat tab (converse + revise) and an Inspect tab
//! (current entity), with an Undo action disabled when no snapshots exist
//! (`UI-18`).

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::character::CharacterBuilderRole;
use lib_soulfire::ids::{CharacterId, WorldBlueprintId};
use lib_soulfire::world::WorldBuilderRole;

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Chat,
    Inspect,
}

/// The conversational character builder (`CHAR-6`).
#[component]
pub fn CharacterBuilder(character_id: CharacterId) -> Element {
    data::subscribe();
    let app = current_app();
    let Some(character) = app.store.character(&character_id).ok().flatten() else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "Character not found." } };
    };
    let session = app
        .store
        .character_builder_session(&character_id)
        .ok()
        .flatten()
        .unwrap_or_default();
    let messages: Vec<(bool, String)> = session
        .messages
        .iter()
        .map(|m| (matches!(m.role, CharacterBuilderRole::User), m.content.clone()))
        .collect();
    let has_snapshots = !session.snapshots.is_empty();

    let send = {
        let app = app.clone();
        let id = character_id.clone();
        use_callback(move |msg: String| {
            let app = app.clone();
            let id = id.clone();
            spawn(async move {
                if let Err(e) = app.character.builder_send(&id, &msg).await {
                    ToastService::error(&format!("{e}"));
                }
                data::touch();
            });
        })
    };
    let undo = {
        let app = app.clone();
        let id = character_id.clone();
        use_callback(move |_: ()| {
            let _ = app.character.builder_undo(&id);
            data::touch();
        })
    };
    let edit_id = character_id.clone();

    rsx! {
        BuilderShell {
            title: "Character Builder".to_string(),
            back: Screen::Characters,
            messages,
            has_snapshots,
            on_send: move |m| send(m),
            on_undo: move |_| undo(()),
            on_edit: move |_| navigate(Screen::CharacterEditor(Some(edit_id.clone()))),
            inspect: rsx! {
                Inspect { label: "Name".to_string(), value: character.name.to_string() }
                Inspect { label: "Subtitle".to_string(), value: character.subtitle.to_string() }
                Inspect { label: "Description".to_string(), value: character.description.to_string() }
                Inspect { label: "Prompt".to_string(), value: character.prompt.to_string() }
                Inspect { label: "Initial message".to_string(), value: character.initial_message.text().to_string() }
            },
        }
    }
}

/// The conversational world builder (`WORLD-20`).
#[component]
pub fn WorldBuilder(blueprint_id: WorldBlueprintId) -> Element {
    data::subscribe();
    let app = current_app();
    let Some(blueprint) = app.store.blueprint(&blueprint_id).ok().flatten() else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "World not found." } };
    };
    let session = app
        .store
        .world_builder_session(&blueprint_id)
        .ok()
        .flatten()
        .unwrap_or_default();
    let messages: Vec<(bool, String)> = session
        .messages
        .iter()
        .map(|m| (matches!(m.role, WorldBuilderRole::User), m.content.clone()))
        .collect();
    let has_snapshots = !session.snapshots.is_empty();

    let send = {
        let app = app.clone();
        let id = blueprint_id.clone();
        use_callback(move |msg: String| {
            let app = app.clone();
            let id = id.clone();
            spawn(async move {
                if let Err(e) = app.world_builder.builder_send(&id, &msg).await {
                    ToastService::error(&format!("{e}"));
                }
                data::touch();
            });
        })
    };
    let undo = {
        let app = app.clone();
        let id = blueprint_id.clone();
        use_callback(move |_: ()| {
            let _ = app.world_builder.builder_undo(&id);
            data::touch();
        })
    };
    let edit_id = blueprint_id.clone();

    rsx! {
        BuilderShell {
            title: "World Builder".to_string(),
            back: Screen::WorldsHome,
            messages,
            has_snapshots,
            on_send: move |m| send(m),
            on_undo: move |_| undo(()),
            on_edit: move |_| navigate(Screen::WorldEditor(Some(edit_id.clone()))),
            inspect: rsx! {
                Inspect { label: "Title".to_string(), value: blueprint.title.to_string() }
                Inspect { label: "Description".to_string(), value: blueprint.description.to_string() }
                Inspect { label: "World prompt".to_string(), value: blueprint.world_prompt.to_string() }
            },
        }
    }
}

/// Shared builder layout: tabbed Chat/Inspect, message log, composer, Undo, and a
/// cross-link to the manual editor (`UI-18`).
#[component]
fn BuilderShell(
    title: String,
    back: Screen,
    messages: Vec<(bool, String)>,
    has_snapshots: bool,
    on_send: EventHandler<String>,
    on_undo: EventHandler<()>,
    on_edit: EventHandler<()>,
    inspect: Element,
) -> Element {
    let mut tab = use_signal(|| Tab::Chat);
    let mut input = use_signal(String::new);
    let mut busy = use_signal(|| false);

    let do_send = use_callback(move |_: ()| {
        let m = input().trim().to_string();
        if m.is_empty() || busy() {
            return;
        }
        input.set(String::new());
        on_send.call(m);
    });

    rsx! {
        div { class: "max-w-3xl mx-auto w-full px-4 py-4 pb-24 md:pb-4 flex flex-col h-[calc(100vh-3.5rem)]",
            // Header.
            div { class: "flex items-center justify-between mb-3",
                button {
                    class: "text-secondary-text text-sm hover:underline",
                    onclick: move |_| navigate(back.clone()),
                    "← Back"
                }
                h1 { class: "text-xl font-serif text-primary-text", "{title}" }
                button {
                    class: "text-link text-sm hover:underline",
                    onclick: move |_| on_edit.call(()),
                    "Open editor →"
                }
            }
            // Tabs.
            div { class: "flex gap-2 mb-3 border-b border-border",
                BTab { active: tab() == Tab::Chat, label: "Chat".to_string(), onclick: move |_| tab.set(Tab::Chat) }
                BTab { active: tab() == Tab::Inspect, label: "Inspect".to_string(), onclick: move |_| tab.set(Tab::Inspect) }
                div { class: "flex-1" }
                button {
                    class: "px-3 py-1.5 text-sm rounded-lg border border-border text-secondary-text disabled:opacity-40 hover-highlight",
                    disabled: !has_snapshots,
                    onclick: move |_| on_undo.call(()),
                    "Undo"
                }
            }

            match tab() {
                Tab::Chat => rsx! {
                    div { class: "flex-1 overflow-y-auto scrollbar-premium",
                        if messages.is_empty() {
                            p { class: "text-center text-secondary-text py-10 font-serif",
                                "Describe what you want to create, and I'll help build it."
                            }
                        }
                        for (is_user, content) in messages.clone() {
                            div { class: if is_user { "flex justify-end my-2" } else { "flex justify-start my-2" },
                                div {
                                    class: if is_user { "max-w-[80%] bg-primary text-primary-text rounded-2xl rounded-br-sm px-4 py-2" } else { "max-w-[80%] bg-surface border border-border text-primary-text rounded-2xl rounded-bl-sm px-4 py-2" },
                                    "{content}"
                                }
                            }
                        }
                    }
                    div { class: "flex gap-2 items-end bg-surface border border-border rounded-2xl p-2 mt-2",
                        textarea {
                            class: "flex-1 bg-transparent resize-none outline-none px-2 py-2 text-primary-text max-h-32",
                            rows: "1",
                            placeholder: "Describe or refine…",
                            value: "{input}",
                            disabled: busy(),
                            oninput: move |e| input.set(e.value()),
                            onkeydown: move |e| if e.key() == Key::Enter && !e.modifiers().shift() {
                                e.prevent_default();
                                do_send(());
                            },
                        }
                        button {
                            class: "crm-primary-button px-4 py-2 rounded-xl disabled:opacity-40",
                            disabled: busy() || input().trim().is_empty(),
                            onclick: move |_| do_send(()),
                            "Send"
                        }
                    }
                },
                Tab::Inspect => rsx! {
                    div { class: "flex-1 overflow-y-auto scrollbar-premium", {inspect.clone()} }
                },
            }
        }
    }
}

#[component]
fn BTab(active: bool, label: String, onclick: EventHandler<MouseEvent>) -> Element {
    let cls = if active { "border-primary text-primary" } else { "border-transparent text-secondary-text" };
    rsx! {
        button { class: "px-4 py-2 -mb-px border-b-2 font-medium {cls}", onclick: move |e| onclick.call(e), "{label}" }
    }
}

#[component]
fn Inspect(label: String, value: String) -> Element {
    rsx! {
        div { class: "mb-4",
            p { class: "text-xs uppercase text-secondary-text mb-1", "{label}" }
            p { class: "text-sm text-primary-text whitespace-pre-wrap bg-surface border border-border rounded-lg p-3",
                if value.is_empty() { "—" } else { "{value}" }
            }
        }
    }
}
