//! The characters list (`UI-17`) and the immersive 1:1 chat screen (`UI-14`,
//! `UI-15`, `UI-16`).

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::chat::{AI_REACTOR, ALLOWED_EMOJIS, ChatMessage, PLAYER_REACTOR};
use lib_soulfire::ids::{CharacterId, ChatId};

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

use super::{EmptyState, Page};

/// How many list rows to fetch per page (keyset paging, `UI-22`).
const PAGE: u32 = 30;

/// The characters & chats list (`UI-17`).
#[component]
pub fn Characters() -> Element {
    use lib_soulfire::character::Character;
    let app = current_app();
    let mut search = use_signal(String::new);
    // Accumulated, already-loaded rows. Each "Load more" appends one keyset page
    // rather than re-fetching the whole prefix.
    let mut items = use_signal(Vec::<Character>::new);
    let mut more = use_signal(|| false);

    // (Re)load the first page whenever the search text or store data changes.
    {
        let app = app.clone();
        use_effect(move || {
            data::subscribe(); // re-run on any store mutation (create/delete)
            let q = search();
            let query = if q.trim().is_empty() {
                None
            } else {
                Some(q.trim())
            };
            let page = app
                .store
                .list_characters(query, None, PAGE)
                .unwrap_or_default();
            more.set(page.len() as u32 == PAGE);
            items.set(page);
        });
    }

    let load_more = {
        let app = app.clone();
        use_callback(move |_: ()| {
            let q = search();
            let query = if q.trim().is_empty() {
                None
            } else {
                Some(q.trim())
            };
            let cursor = items.read().last().cloned();
            let page = app
                .store
                .list_characters(query, cursor.as_ref(), PAGE)
                .unwrap_or_default();
            more.set(page.len() as u32 == PAGE);
            items.write().extend(page);
        })
    };

    let list = items();
    rsx! {
        Page { title: "Characters".to_string(),
            div { class: "flex justify-end gap-2 mb-3",
                button {
                    class: "px-4 py-2 rounded-lg text-sm border border-border text-primary-text hover-highlight",
                    onclick: move |_| {
                        use lib_soulfire::character::{Character, InitialMessage};
                        use lib_soulfire::strings::CharacterName;
                        let c = Character::builder()
                            .name(CharacterName::coerce("New Character"))
                            .initial_message(InitialMessage::default())
                            .build();
                        let _ = app.store.save_character(&c);
                        data::touch();
                        navigate(Screen::CharacterBuilder(c.character_id));
                    },
                    "✨ Character Builder"
                }
                button {
                    class: "crm-primary-button px-4 py-2 rounded-lg text-sm",
                    onclick: move |_| navigate(Screen::CharacterEditor(None)),
                    "+ New Character"
                }
            }
            input {
                class: "input-premium w-full mb-3",
                placeholder: "Search characters…",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
            if list.is_empty() {
                EmptyState { message: "No characters yet. Create one to start chatting.".to_string() }
            } else {
                div { class: "flex flex-col gap-2",
                    for c in list.clone() {
                        CharacterRow {
                            character: c,
                            on_delete: move |id: CharacterId| {
                                items.write().retain(|c| c.character_id != id);
                            },
                        }
                    }
                }
                if more() {
                    button {
                        class: "mt-3 w-full py-2 rounded-lg border border-border text-secondary-text hover-highlight text-sm",
                        onclick: move |_| load_more(()),
                        "Load more"
                    }
                }
            }
        }
    }
}

#[component]
fn CharacterRow(
    character: lib_soulfire::character::Character,
    /// Called with the deleted id so the parent can drop just this row from its
    /// loaded list, without re-fetching the page (`UI-22`).
    on_delete: EventHandler<CharacterId>,
) -> Element {
    use soulfire_core::store::ImageOwnerKind;
    let app = current_app();
    let avatar = character
        .image
        .and_then(|i| i.emoji())
        .unwrap_or("🙂")
        .to_string();
    let portrait = character.portrait.and_then(|_| {
        crate::image_util::data_uri(
            &app,
            ImageOwnerKind::Character,
            &character.character_id.to_string(),
        )
    });
    let cid_open = character.character_id.clone();
    let cid_edit = character.character_id.clone();
    let cid_del = character.character_id.clone();
    let subtitle = character.subtitle.to_string();
    let mut confirming = use_signal(|| false);
    rsx! {
        div { class: "flex items-center gap-3 bg-surface border border-border rounded-xl p-3",
            if let Some(uri) = portrait {
                img { class: "w-12 h-12 rounded-full object-cover shrink-0", src: "{uri}" }
            } else {
                div { class: "w-12 h-12 rounded-full bg-primary-lighter flex items-center justify-center text-2xl shrink-0", "{avatar}" }
            }
            button {
                class: "flex-1 text-left min-w-0",
                onclick: move |_| navigate(Screen::Chat(cid_open.clone())),
                p { class: "font-semibold text-primary-text truncate", "{character.name}" }
                if !subtitle.is_empty() {
                    p { class: "text-sm text-secondary-text truncate", "{subtitle}" }
                }
            }
            button {
                class: "px-3 py-1.5 rounded-lg border border-border text-secondary-text text-sm hover-highlight",
                onclick: move |_| navigate(Screen::CharacterEditor(Some(cid_edit.clone()))),
                "Edit"
            }
            button {
                class: "px-3 py-1.5 rounded-lg border border-border text-secondary-text text-sm hover-highlight",
                onclick: move |_| confirming.set(true),
                "🗑"
            }
            crate::components::ConfirmDialog {
                open: confirming(),
                title: "Delete character?".to_string(),
                message: "This permanently deletes the character and its chat.".to_string(),
                danger: true,
                confirm_label: "Delete".to_string(),
                confirm_word: Some("delete".to_string()),
                on_confirm: move |_| {
                    if app.store.delete_character(&cid_del).is_ok() {
                        on_delete.call(cid_del.clone());
                    }
                    confirming.set(false);
                },
                on_cancel: move |_| confirming.set(false),
            }
        }
    }
}

/// The immersive chat screen (`UI-14`).
#[component]
pub fn Chat(character_id: CharacterId) -> Element {
    data::subscribe();
    let app = current_app();
    let character = app.store.character(&character_id).ok().flatten();

    // Open (or create) the chat for this character once (CHAT-1/2).
    let app_open = app.clone();
    let cid_open = character_id.clone();
    let opened = use_resource(move || {
        let app = app_open.clone();
        let cid = cid_open.clone();
        async move { app.chat.open_chat(&cid).await.ok().map(|c| c.chat_id) }
    });

    let Some(character) = character else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "Character not found." } };
    };
    let name = character.name.to_string();
    let avatar = character
        .image
        .and_then(|i| i.emoji())
        .unwrap_or("🙂")
        .to_string();
    let portrait = character.portrait.and_then(|_| {
        crate::image_util::data_uri(
            &app,
            soulfire_core::store::ImageOwnerKind::Character,
            &character_id.to_string(),
        )
    });

    let chat_id = opened.read().clone().flatten();

    rsx! {
        div { class: "relative min-h-screen flex flex-col",
            style: "background: radial-gradient(120% 80% at 50% 0%, var(--color-primary-darkest), var(--color-background));",
            // Header pill (UI-14).
            div { class: "sticky top-0 z-20 flex items-center gap-3 px-4 py-3",
                button {
                    class: "px-3 py-2 rounded-full bg-surface/70 backdrop-blur border border-border text-primary-text",
                    onclick: move |_| navigate(Screen::Characters),
                    "← Back"
                }
                div { class: "flex items-center gap-2 px-4 py-2 rounded-full bg-surface/70 backdrop-blur border border-border",
                    if let Some(uri) = portrait {
                        img { class: "w-6 h-6 rounded-full object-cover", src: "{uri}" }
                    } else {
                        span { class: "text-xl", "{avatar}" }
                    }
                    span { class: "font-serif text-primary-text", "{name}" }
                }
            }

            match chat_id {
                Some(cid) => rsx! { ChatBody { chat_id: cid } },
                None => rsx! {
                    div { class: "flex-1 flex items-center justify-center text-secondary-text", "Opening chat…" }
                },
            }
        }
    }
}

#[component]
fn ChatBody(chat_id: ChatId) -> Element {
    use lib_soulfire::draft::{Draft, DraftScope};
    use lib_soulfire::strings::DraftContent;
    data::subscribe();
    let app = current_app();
    let messages = app.store.chat_messages(&chat_id).unwrap_or_default();
    let scope = DraftScope::Chat {
        chat_id: chat_id.clone(),
    };

    // Restore any in-progress message draft (DATA-26, UI-14).
    let restored = app
        .store
        .draft_for_scope(&scope)
        .ok()
        .flatten()
        .map(|d| d.content.to_string())
        .unwrap_or_default();
    let mut input = use_signal(|| restored);
    let mut streaming = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let app_draft = app.clone();
    let id_draft = chat_id.clone();

    let send = use_callback(move |_: ()| {
        let text = input().trim().to_string();
        if text.is_empty() || busy() {
            return;
        }
        input.set(String::new());
        let _ = app.store.delete_draft_for_scope(&DraftScope::Chat {
            chat_id: chat_id.clone(),
        });
        let app = app.clone();
        let cid = chat_id.clone();
        busy.set(true);
        streaming.set(String::new());
        spawn(async move {
            let on_delta = move |d: &str| streaming.with_mut(|s| s.push_str(d));
            if let Err(e) = app.chat.send_message(&cid, &text, on_delta).await {
                ToastService::error(&format!("{e}"));
            }
            streaming.set(String::new());
            busy.set(false);
            data::touch();
        });
    });

    rsx! {
        div { class: "flex-1 overflow-y-auto scrollbar-premium px-4 pb-40 max-w-2xl mx-auto w-full",
            for msg in messages.clone() { ChatBubble { message: msg } }
            if !streaming().is_empty() {
                div { class: "flex justify-start my-2",
                    div { class: "max-w-[80%] bg-surface border border-border rounded-2xl rounded-bl-sm px-4 py-2 text-primary-text whitespace-pre-wrap", "{streaming}" }
                }
            }
            if busy() && streaming().is_empty() {
                div { class: "flex justify-start my-2",
                    div { class: "bg-surface border border-border rounded-2xl px-4 py-2 text-secondary-text", "…" }
                }
            }
        }
        // Composer.
        div { class: "fixed bottom-0 inset-x-0 z-20 p-4",
            div { class: "max-w-2xl mx-auto flex gap-2 items-end bg-surface/80 backdrop-blur border border-border rounded-2xl p-2",
                textarea {
                    class: "flex-1 bg-transparent resize-none outline-none px-2 py-2 text-primary-text max-h-32",
                    rows: "1",
                    placeholder: "Message…",
                    value: "{input}",
                    disabled: busy(),
                    oninput: move |e| {
                        input.set(e.value());
                        let d = Draft::builder()
                            .scope(DraftScope::Chat { chat_id: id_draft.clone() })
                            .content(DraftContent::coerce(&e.value()))
                            .build();
                        let _ = app_draft.store.save_draft(&d);
                    },
                    onkeydown: move |e| if e.key() == Key::Enter && !e.modifiers().shift() {
                        e.prevent_default();
                        send(());
                    },
                }
                button {
                    class: "crm-primary-button px-4 py-2 rounded-xl disabled:opacity-40",
                    disabled: busy() || input().trim().is_empty(),
                    onclick: move |_| send(()),
                    "Send"
                }
            }
        }
    }
}

#[component]
fn ChatBubble(message: ChatMessage) -> Element {
    let app = current_app();
    let is_player = message.sender.is_player();
    let align = if is_player {
        "justify-end"
    } else {
        "justify-start"
    };
    let bubble = if is_player {
        "bg-primary text-primary-text rounded-br-sm"
    } else {
        "bg-surface border border-border text-primary-text rounded-bl-sm"
    };
    let mut show_picker = use_signal(|| false);
    let text = message.message.to_string();
    let msg_for_react = message.clone();

    rsx! {
        div { class: "flex {align} my-2",
            div { class: "max-w-[80%]",
                div {
                    class: "rounded-2xl px-4 py-2 {bubble}",
                    onclick: move |_| show_picker.toggle(),
                    sp_markdown::ChatMarkdown { text, class: String::new() }
                }
                // Reactions under the bubble (DATA-6, UI-15).
                if !message.emoji_reactions.is_empty() {
                    div { class: "flex gap-1 mt-1 px-1",
                        for (_, emoji) in message.emoji_reactions.iter() {
                            span { class: "text-sm", "{emoji}" }
                        }
                    }
                }
                // Emoji picker limited to the allowed set (UI-15).
                if show_picker() {
                    div { class: "flex gap-1 mt-1 bg-surface border border-border rounded-full px-2 py-1 w-fit",
                        for emoji in ALLOWED_EMOJIS {
                            {
                                let app = app.clone();
                                let base = msg_for_react.clone();
                                rsx! {
                                    button {
                                        class: "text-lg hover:scale-125 transition-transform",
                                        onclick: move |_| {
                                            let mut m = base.clone();
                                            m.emoji_reactions.set(PLAYER_REACTOR, emoji);
                                            let _ = app.store.save_chat_message(&m);
                                            show_picker.set(false);
                                            data::touch();
                                        },
                                        "{emoji}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Silence unused-import lints for reactor constants used conditionally above.
#[allow(unused)]
fn _reactor_keys() -> (&'static str, &'static str) {
    (PLAYER_REACTOR, AI_REACTOR)
}
