//! The root component, the lock/first-run gate (`SEC-5`, `ONB-1`), and the app
//! shell with primary navigation (`UI-4`, `UI-5`, `UI-6`).

use std::str::FromStr;

use dioxus::prelude::*;

use lib_soulfire::ai_model::AiVendor;
use lib_soulfire::credentials::ProviderCredential;

use soulfire_core::seed::seed_starter_worlds;
use soulfire_core::store::Store;

use crate::nav::{Destination, SCREEN, Screen, navigate};
use crate::state::{AppContext, data_dir, is_initialized};
use crate::theme::theme_class;
use crate::{data, screens};

/// The root: loads the stylesheet and gates the app behind unlock (`UI-23`).
#[component]
pub fn App() -> Element {
    let ctx = use_signal(|| None::<AppContext>);
    use_context_provider(|| ctx);
    // True while a first-time user is in the onboarding story flow (ONB-2).
    let onboarding = use_signal(|| false);
    use_context_provider(|| onboarding);

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        sp_ui::components::toastcontainer::ToastContainer {}
        if ctx().is_some() {
            Shell {}
        } else {
            LockScreen {}
        }
    }
}

/// The launch/lock screen: first-run setup or master-password unlock (`SEC-4`,
/// `SEC-5`, `ONB-1`).
#[component]
fn LockScreen() -> Element {
    let mut ctx = use_context::<Signal<Option<AppContext>>>();
    let mut onboarding = use_context::<Signal<bool>>();
    let first_run = use_hook(is_initialized);
    let first_run = !first_run; // is_initialized() == false => first run

    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut api_key = use_signal(String::new);
    let mut error = use_signal(|| Option::<String>::None);

    let submit = use_callback(move |_: ()| {
        error.set(None);
        let pw = password();
        if pw.is_empty() {
            error.set(Some("Enter your master password.".into()));
            return;
        }
        if first_run {
            if pw != confirm() {
                error.set(Some("Passwords do not match.".into()));
                return;
            }
            match Store::initialize(data_dir(), &pw) {
                Ok(store) => {
                    // Save the OpenAI key if provided (ONB-1, AI-3).
                    let key = api_key();
                    if !key.trim().is_empty() {
                        let _ = store.save_credential(&ProviderCredential::new(
                            AiVendor::OpenAI,
                            key.trim().to_string(),
                        ));
                    }
                    // Seed starter worlds on first launch (ONB-5). `first_run_completed`
                    // is set when the onboarding story finishes (ONB-2/4), not here.
                    let _ = seed_starter_worlds(&store, &soulfire_core::clock::SystemClock);
                    onboarding.set(true);
                    ctx.set(Some(AppContext::new(store)));
                }
                Err(e) => error.set(Some(format!("Could not create store: {e}"))),
            }
        } else {
            match Store::unlock(data_dir(), &pw) {
                Ok(store) => ctx.set(Some(AppContext::new(store))),
                Err(_) => error.set(Some("Incorrect master password.".into())),
            }
        }
    });

    rsx! {
        div { class: "dark",
            div { class: "min-h-screen bg-background text-primary-text flex items-center justify-center p-6 bg-premium-pattern",
                div { class: "w-full max-w-md bg-surface border border-border rounded-2xl shadow-2xl p-8",
                    h1 { class: "text-3xl font-serif text-center mb-2 text-glow text-primary", "Soulfire" }
                    p { class: "text-center text-secondary-text mb-6",
                        if first_run { "Welcome. Set a master password to protect your stories." }
                        else { "Welcome back. Unlock to continue." }
                    }

                    if let Some(err) = error() {
                        div { class: "mb-4 text-sm text-error text-center", "{err}" }
                    }

                    label { class: "block text-sm font-semibold text-primary-light mb-1", "Master password" }
                    input {
                        class: "input-premium w-full mb-3",
                        r#type: "password",
                        value: "{password}",
                        oninput: move |e| password.set(e.value()),
                        onkeydown: move |e| if e.key() == Key::Enter { submit(()); },
                    }

                    if first_run {
                        label { class: "block text-sm font-semibold text-primary-light mb-1", "Confirm password" }
                        input {
                            class: "input-premium w-full mb-3",
                            r#type: "password",
                            value: "{confirm}",
                            oninput: move |e| confirm.set(e.value()),
                        }
                        label { class: "block text-sm font-semibold text-primary-light mb-1", "OpenAI API key (optional now)" }
                        input {
                            class: "input-premium w-full mb-3",
                            r#type: "password",
                            placeholder: "sk-…",
                            value: "{api_key}",
                            oninput: move |e| api_key.set(e.value()),
                        }
                        p { class: "text-xs text-secondary-text mb-4",
                            "There is no password recovery — if you lose it, your data cannot be unlocked."
                        }
                    }

                    button {
                        class: "crm-primary-button w-full py-3 rounded-lg font-semibold",
                        onclick: move |_| { submit(()); },
                        if first_run { "Create & enter" } else { "Unlock" }
                    }
                }
            }
        }
    }
}

/// The unlocked app shell: themed surface, title bar / nav, and the active
/// screen (`UI-4`, `UI-5`, `UI-6`).
#[component]
fn Shell() -> Element {
    let ctx_sig = use_context::<Signal<Option<AppContext>>>();
    let app = ctx_sig().expect("shell requires an unlocked context");
    use_context_provider(|| app.clone());
    let onboarding = use_context::<Signal<bool>>();

    data::subscribe();
    let theme = app.store.app_settings().map(|s| s.color_theme).unwrap_or_default();
    let immersive = SCREEN.read().is_immersive() || onboarding();

    rsx! {
        div { class: "{theme_class(theme)} bg-background text-primary-text min-h-screen",
            if onboarding() {
                {screens::onboarding::render_first_run()}
            } else if immersive {
                // Immersive surfaces hide all chrome (UI-4).
                {screens::render_active()}
            } else {
                div { class: "flex min-h-screen",
                    Sidebar {}
                    div { class: "flex-1 flex flex-col min-w-0",
                        TitleBar {}
                        main { class: "flex-1 overflow-y-auto scrollbar-premium", {screens::render_active()} }
                    }
                }
                BottomNav {}
            }
        }
    }
}

/// Desktop left sidebar (`UI-6`).
#[component]
fn Sidebar() -> Element {
    let current = SCREEN.read().destination();
    rsx! {
        nav { class: "hidden md:flex flex-col w-56 shrink-0 bg-sidebar/0 border-r border-border p-4 gap-1",
            div { class: "px-2 py-4 text-2xl font-serif text-primary text-glow", "Soulfire" }
            NavItem { dest: Destination::Worlds, active: current == Destination::Worlds, icon: "🌍", label: "Worlds" }
            NavItem { dest: Destination::Characters, active: current == Destination::Characters, icon: "💬", label: "Characters" }
            NavItem { dest: Destination::Settings, active: current == Destination::Settings, icon: "⚙\u{fe0f}", label: "Settings" }
        }
    }
}

/// Mobile bottom navigation (`UI-6`).
#[component]
fn BottomNav() -> Element {
    let current = SCREEN.read().destination();
    rsx! {
        nav { class: "md:hidden fixed bottom-0 inset-x-0 z-40 flex bg-surface border-t border-border",
            style: "padding-bottom: env(safe-area-inset-bottom);",
            BottomItem { dest: Destination::Worlds, active: current == Destination::Worlds, icon: "🌍", label: "Worlds" }
            BottomItem { dest: Destination::Characters, active: current == Destination::Characters, icon: "💬", label: "Characters" }
            BottomItem { dest: Destination::Settings, active: current == Destination::Settings, icon: "⚙\u{fe0f}", label: "Settings" }
        }
    }
}

#[component]
fn NavItem(dest: Destination, active: bool, icon: String, label: String) -> Element {
    let cls = if active {
        "bg-primary-lighter text-primary"
    } else {
        "text-secondary-text hover-highlight"
    };
    rsx! {
        button {
            class: "flex items-center gap-3 px-3 py-2.5 rounded-lg text-left font-medium transition-colors {cls}",
            onclick: move |_| navigate(dest_to_screen(dest)),
            span { class: "text-lg", "{icon}" }
            span { "{label}" }
        }
    }
}

#[component]
fn BottomItem(dest: Destination, active: bool, icon: String, label: String) -> Element {
    let cls = if active { "text-primary" } else { "text-secondary-text" };
    rsx! {
        button {
            class: "flex-1 flex flex-col items-center justify-center py-2 gap-0.5 min-h-12 {cls}",
            onclick: move |_| navigate(dest_to_screen(dest)),
            span { class: "text-lg", "{icon}" }
            span { class: "text-xs", "{label}" }
        }
    }
}

/// The standard-page title bar (`UI-5`).
#[component]
fn TitleBar() -> Element {
    rsx! {
        header { class: "titlebar-gradient sticky top-0 z-30 flex items-center justify-between px-4 h-14 border-b border-border backdrop-blur",
            button {
                class: "md:hidden text-xl font-serif text-primary",
                onclick: move |_| navigate(Screen::WorldsHome),
                "Soulfire"
            }
            div { class: "flex-1" }
            button {
                class: "w-9 h-9 rounded-full bg-primary-lighter text-primary flex items-center justify-center",
                onclick: move |_| navigate(Screen::Profile),
                "🙂"
            }
        }
    }
}

fn dest_to_screen(dest: Destination) -> Screen {
    match dest {
        Destination::Worlds => Screen::WorldsHome,
        Destination::Characters => Screen::Characters,
        Destination::Settings => Screen::Settings,
    }
}

// Re-export for use in screens.
pub fn current_app() -> AppContext {
    use_context::<AppContext>()
}

/// Parse a non-empty trimmed string, returning a UI error message on failure.
pub fn parse_or_err<T: FromStr>(s: &str, what: &str) -> Result<T, String> {
    T::from_str(s.trim()).map_err(|_| format!("Invalid {what}."))
}
