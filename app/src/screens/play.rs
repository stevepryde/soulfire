//! The immersive adventure play screen (`UI-10`..`UI-13`).

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::ids::AdventureId;
use lib_soulfire::world::{AdventureMessage, AdventureMessageType, GmProposal};

use lib_soulfire::draft::{Draft, DraftScope};
use lib_soulfire::strings::DraftContent;

use soulfire_core::world::TurnOutcome;

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

#[component]
pub fn Play(adventure_id: AdventureId) -> Element {
    data::subscribe();
    let app = current_app();
    let id = adventure_id.clone();
    let scope = DraftScope::Adventure { adventure_id: adventure_id.clone() };

    let Some(adventure) = app.store.adventure(&id).ok().flatten() else {
        return rsx! { div { class: "p-8 text-center text-secondary-text", "Adventure not found." } };
    };
    let messages = app.store.adventure_messages(&id).unwrap_or_default();
    let pending = app.store.pending_gm_proposals(&id).unwrap_or_default();

    // Restore any in-progress action draft (DATA-26, UI-12).
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
    let mut status = use_signal(|| "What do you do?".to_string());
    let mut npc_open = use_signal(|| false);
    let mut npc_name = use_signal(String::new);
    let mut extracting = use_signal(|| false);

    let app_draft = app.clone();
    let id_draft = adventure_id.clone();

    // Extraction is offered once the adventure has progressed (CHAR-10, UI-10).
    let progressed = !messages.is_empty();
    let extract = {
        let app = app.clone();
        let id = id.clone();
        use_callback(move |_: ()| {
            let name = npc_name().trim().to_string();
            if name.is_empty() || extracting() {
                return;
            }
            let app = app.clone();
            let id = id.clone();
            extracting.set(true);
            spawn(async move {
                match app.character.extract_npc(&id, &name).await {
                    Ok(c) => {
                        ToastService::info(&format!("{name} is now a character you can chat with."));
                        npc_open.set(false);
                        npc_name.set(String::new());
                        data::touch();
                        navigate(Screen::Chat(c.character_id));
                    }
                    Err(e) => ToastService::error(&format!("Could not extract {name}: {e}")),
                }
                extracting.set(false);
            });
        })
    };

    let title = adventure
        .world_title
        .as_ref()
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Adventure".to_string());
    let watermark = adventure.world_image.map(|i| i.emoji()).unwrap_or("🌊").to_string();

    let send = use_callback(move |_: ()| {
        let action = input().trim().to_string();
        if action.is_empty() || busy() {
            return;
        }
        input.set(String::new());
        let _ = app.store.delete_draft_for_scope(&DraftScope::Adventure { adventure_id: id.clone() });
        let app = app.clone();
        let id = id.clone();
        busy.set(true);
        streaming.set(String::new());
        status.set("The world responds…".to_string());
        spawn(async move {
            let on_delta = move |d: &str| streaming.with_mut(|s| s.push_str(d));
            match app.world.take_turn(&id, &action, on_delta).await {
                Ok(TurnOutcome::Narration { state_update_failed, .. }) => {
                    if state_update_failed {
                        ToastService::warn("The world's memory didn't fully update; it will self-correct.");
                    }
                }
                Ok(TurnOutcome::Warning(w)) => ToastService::warn(&w),
                Ok(TurnOutcome::GmAnswer { .. }) | Ok(TurnOutcome::GmProposal { .. }) => {}
                Err(e) => ToastService::error(&format!("{e}")),
            }
            streaming.set(String::new());
            status.set("What do you do?".to_string());
            busy.set(false);
            data::touch();
        });
    });

    rsx! {
        div { class: "relative min-h-screen flex flex-col",
            style: "background: radial-gradient(120% 80% at 50% 0%, var(--color-primary-darkest), var(--color-background));",
            // Faint emoji watermark backdrop (UI-10).
            div { class: "pointer-events-none fixed inset-0 flex items-center justify-center text-[20rem] opacity-[0.04] select-none", "{watermark}" }

            // Floating header pill (UI-10).
            div { class: "sticky top-0 z-20 flex items-center gap-3 px-4 py-3",
                button {
                    class: "px-3 py-2 rounded-full bg-surface/70 backdrop-blur border border-border text-primary-text",
                    onclick: move |_| navigate(Screen::WorldsHome),
                    "← Back"
                }
                div { class: "px-4 py-2 rounded-full bg-surface/70 backdrop-blur border border-border font-serif text-primary-text", "{title}" }
                div { class: "ml-auto flex gap-2",
                    {
                        let pid = adventure_id.clone();
                        rsx! {
                            button {
                                class: "px-3 py-2 rounded-full bg-surface/70 backdrop-blur border border-border text-primary-text text-sm",
                                onclick: move |_| navigate(Screen::AdventurePrompt(pid.clone())),
                                "📜 Prompt"
                            }
                        }
                    }
                    if progressed {
                        button {
                            class: "px-3 py-2 rounded-full bg-surface/70 backdrop-blur border border-border text-primary-text text-sm",
                            onclick: move |_| npc_open.toggle(),
                            "✨ NPC"
                        }
                    }
                }
            }

            // "Bring a Character to Life" — extract an NPC (CHAR-10).
            if npc_open() {
                div { class: "sticky top-16 z-20 px-4",
                    div { class: "max-w-md mx-auto bg-surface border border-border rounded-xl p-3 flex gap-2 items-center",
                        input {
                            class: "input-premium flex-1",
                            placeholder: "Name an NPC from this story…",
                            value: "{npc_name}",
                            oninput: move |e| npc_name.set(e.value()),
                            onkeydown: move |e| if e.key() == Key::Enter { extract(()); },
                        }
                        button {
                            class: "crm-primary-button px-4 py-2 rounded-lg text-sm disabled:opacity-40",
                            disabled: extracting() || npc_name().trim().is_empty(),
                            onclick: move |_| extract(()),
                            if extracting() { "…" } else { "Bring to life" }
                        }
                    }
                }
            }

            // Message panel.
            div { class: "flex-1 overflow-y-auto scrollbar-premium px-4 pb-40 max-w-3xl mx-auto w-full",
                if messages.is_empty() && streaming().is_empty() {
                    div { class: "text-center py-20",
                        div { class: "text-7xl mb-4", "{watermark}" }
                        p { class: "font-serif text-xl text-secondary-text", "The story is beginning…" }
                    }
                }
                for msg in messages.clone() { MessageView { message: msg } }
                if !streaming().is_empty() {
                    div { class: "font-serif text-lg leading-relaxed text-primary-text my-5 whitespace-pre-wrap", "{streaming}" }
                }
                if busy() && streaming().is_empty() {
                    div { class: "text-secondary-text my-5 font-serif italic", "…" }
                }
                for proposal in pending.clone() { ProposalView { proposal } }
            }

            // Glassy composer (UI-12).
            div { class: "fixed bottom-0 inset-x-0 z-20 p-4",
                div { class: "max-w-3xl mx-auto",
                    p { class: "text-xs text-secondary-text mb-1 px-2", "{status}" }
                    div { class: "flex gap-2 items-end bg-surface/80 backdrop-blur border border-border rounded-2xl p-2",
                        textarea {
                            class: "flex-1 bg-transparent resize-none outline-none px-2 py-2 text-primary-text max-h-32",
                            rows: "1",
                            placeholder: "Type an action, or /gm to ask the game master…",
                            value: "{input}",
                            disabled: busy(),
                            oninput: move |e| {
                                input.set(e.value());
                                let d = Draft::builder()
                                    .scope(DraftScope::Adventure { adventure_id: id_draft.clone() })
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
    }
}

/// Render a turn-log message by type (`UI-11`).
#[component]
fn MessageView(message: AdventureMessage) -> Element {
    let text = message.content.to_string();
    match message.message_type {
        AdventureMessageType::Narration => rsx! {
            div { class: "my-5",
                sp_markdown::Markdown { text, class: "font-serif text-lg leading-relaxed text-primary-text".to_string() }
            }
        },
        AdventureMessageType::UserAction => rsx! {
            div { class: "flex justify-end my-3",
                div { class: "max-w-[80%] bg-primary text-primary-text rounded-2xl rounded-br-sm px-4 py-2",
                    p { class: "text-[10px] uppercase opacity-70 mb-0.5", "You" }
                    p { "{text}" }
                }
            }
        },
        AdventureMessageType::GameMasterRequest => rsx! {
            div { class: "flex justify-end my-3",
                div { class: "max-w-[80%] bg-secondary/30 border border-secondary/40 text-primary-text rounded-2xl rounded-br-sm px-4 py-2",
                    p { class: "text-[10px] uppercase opacity-70 mb-0.5", "GM request" }
                    p { "{text}" }
                }
            }
        },
        AdventureMessageType::GameMasterResponse => rsx! {
            div { class: "flex justify-start my-3",
                div { class: "max-w-[80%] bg-surface border border-border text-primary-text rounded-2xl rounded-bl-sm px-4 py-2",
                    p { class: "text-[10px] uppercase opacity-70 mb-0.5", "Game master" }
                    p { "{text}" }
                }
            }
        },
    }
}

/// A staged `/gm` change proposal with a diff and Accept/Reject (`UI-11`,
/// `WORLD-17`).
#[component]
fn ProposalView(proposal: GmProposal) -> Element {
    let app_accept = current_app();
    let app_reject = app_accept.clone();
    let pid_accept = proposal.proposal_id.clone();
    let pid_reject = proposal.proposal_id.clone();
    rsx! {
        div { class: "my-4 bg-surface border border-primary/40 rounded-xl p-4",
            p { class: "text-sm font-semibold text-primary mb-2", "Proposed change" }
            for entry in proposal.changes.clone() {
                div { class: "text-xs mb-2 font-mono",
                    p { class: "text-secondary-text", "{entry.path}" }
                    if let Some(before) = &entry.before {
                        p { class: "text-error/80 line-through", "− {before}" }
                    }
                    if let Some(after) = &entry.after {
                        p { class: "text-link", "+ {after}" }
                    }
                }
            }
            div { class: "flex gap-2 mt-3",
                button {
                    class: "crm-primary-button px-4 py-1.5 rounded-lg text-sm",
                    onclick: move |_| {
                        let app = app_accept.clone();
                        let pid = pid_accept.clone();
                        spawn(async move {
                            if let Err(e) = app.world.accept_proposal(&pid).await {
                                ToastService::error(&format!("{e}"));
                            }
                            data::touch();
                        });
                    },
                    "Accept"
                }
                button {
                    class: "px-4 py-1.5 rounded-lg text-sm border border-border text-secondary-text",
                    onclick: move |_| {
                        let app = app_reject.clone();
                        let pid = pid_reject.clone();
                        spawn(async move {
                            let _ = app.world.reject_proposal(&pid).await;
                            data::touch();
                        });
                    },
                    "Reject"
                }
            }
        }
    }
}
