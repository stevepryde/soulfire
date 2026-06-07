//! The first-run story: name capture over an atmospheric backdrop, then an
//! auto-launched starter adventure (`ONB-2`, `ONB-3`, `ONB-4`).

use dioxus::prelude::*;
use sp_ui::toast::ToastService;

use lib_soulfire::strings::PlayerName;
use lib_soulfire::world::WorldBlueprint;

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

/// The lead starter seed id (`ONB-5`).
const LEAD_STARTER: &str = "beneath_verath";

pub fn render_first_run() -> Element {
    rsx! { FirstRunStory {} }
}

#[component]
fn FirstRunStory() -> Element {
    let app = current_app();
    let mut onboarding = use_context::<Signal<bool>>();
    let profile = app.store.app_profile().unwrap_or_default();
    let prefill = profile
        .nickname
        .as_ref()
        .or(profile.name.as_ref())
        .map(|n| n.to_string())
        .unwrap_or_default();

    let mut name = use_signal(|| prefill);
    let mut busy = use_signal(|| false);

    let lead = lead_starter(&app);

    // Mark first-run complete so the auto-start never recurs (ONB-2/4).
    let complete = {
        let app = app.clone();
        use_callback(move |_: ()| {
            let mut install = app.store.install_state().unwrap_or_default();
            install.first_run_completed = true;
            let _ = app.store.save_install_state(&install);
        })
    };

    let begin = {
        let app = app.clone();
        let lead = lead.clone();
        use_callback(move |save_name: Option<String>| {
            if busy() {
                return;
            }
            // Save the adventurer name to the player profile (ONB-3, DATA-17).
            if let Some(n) = save_name.filter(|n| !n.trim().is_empty()) {
                let mut player = app.store.player_profile().unwrap_or_default();
                player.player_name = PlayerName::coerce(n.trim());
                let _ = app.store.save_player_profile(&player);
            }
            complete(());

            let Some(bp) = lead.clone() else {
                // No starter available → land on home with a friendly note (ONB-4).
                onboarding.set(false);
                navigate(Screen::WorldsHome);
                return;
            };
            if !app.has_api_key() {
                ToastService::info("Add your OpenAI key in Settings to begin a story.");
                onboarding.set(false);
                navigate(Screen::WorldsHome);
                return;
            }
            busy.set(true);
            let app = app.clone();
            spawn(async move {
                match app.world.start_adventure(&bp, |_| {}).await {
                    Ok(adv) => {
                        data::touch();
                        onboarding.set(false);
                        navigate(Screen::Play(adv.adventure_id));
                    }
                    Err(e) => {
                        ToastService::error(&format!("Could not begin: {e}"));
                        onboarding.set(false);
                        navigate(Screen::WorldsHome);
                    }
                }
                busy.set(false);
            });
        })
    };

    let world_emoji = lead
        .as_ref()
        .and_then(|b| b.image)
        .map(|i| i.emoji())
        .unwrap_or("🌊")
        .to_string();
    let world_desc = lead
        .as_ref()
        .map(|b| b.description.to_string())
        .unwrap_or_default();

    rsx! {
        div { class: "dark",
            div { class: "min-h-screen flex flex-col items-center justify-center p-6 text-center",
                style: "background: radial-gradient(120% 80% at 50% 0%, var(--color-primary-darkest), #050505);",
                div { class: "text-7xl mb-6 opacity-90", "{world_emoji}" }
                if busy() {
                    p { class: "font-serif text-2xl text-primary-text mb-2", "Your story is beginning…" }
                    p { class: "font-serif text-secondary-text max-w-md", "{world_desc}" }
                } else {
                    h1 { class: "font-serif text-3xl md:text-4xl text-primary-text mb-8", "What shall we call you?" }
                    input {
                        class: "input-premium w-full max-w-sm text-center text-lg mb-6",
                        placeholder: "Your name…",
                        value: "{name}",
                        autofocus: true,
                        oninput: move |e| name.set(e.value()),
                        onkeydown: move |e| if e.key() == Key::Enter { begin(Some(name())); },
                    }
                    div { class: "flex flex-col items-center gap-3",
                        button {
                            class: "crm-primary-button px-10 py-3 rounded-xl text-lg",
                            onclick: move |_| begin(Some(name())),
                            "Continue"
                        }
                        button {
                            class: "text-secondary-text text-sm hover:underline",
                            onclick: move |_| begin(None),
                            "Skip"
                        }
                        button {
                            class: "text-secondary-text/70 text-xs hover:underline mt-4",
                            onclick: move |_| {
                                complete(());
                                onboarding.set(false);
                                navigate(Screen::WorldsHome);
                            },
                            "Browse other worlds instead"
                        }
                    }
                }
            }
        }
    }
}

/// The lead starter blueprint, via the seed ledger (`ONB-5`).
fn lead_starter(app: &crate::state::AppContext) -> Option<WorldBlueprint> {
    let install = app.store.install_state().ok()?;
    let id = install.starter(LEAD_STARTER)?.blueprint_id.clone()?;
    app.store.blueprint(&id).ok().flatten()
}
