//! The root component, the lock/first-run gate (`SEC-5`, `ONB-1`), and the app
//! shell with primary navigation (`UI-4`, `UI-5`, `UI-6`).

use dioxus::prelude::*;

use lib_soulfire::ai_model::AiVendor;
use lib_soulfire::credentials::ProviderCredential;

use soulfire_core::seed::seed_starter_worlds;
use soulfire_core::store::Store;

use crate::nav::{Destination, SCREEN, Screen, navigate};
use crate::state::{AppContext, data_dir, is_initialized};
use crate::theme::theme_class;
use crate::{data, screens};

/// App icon shown in the title bar (ported from Soulfire-OG).
const APP_ICON: Asset = asset!("/assets/images/app-icon.png");
/// Placeholder avatar used for the profile button.
const MISSING_PROFILE: Asset = asset!("/assets/images/missingprofile512.png");

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

/// The unlocked app shell: themed surface, title bar, desktop sidebar / mobile
/// bottom nav, and the active screen — ported from Soulfire-OG's `AppLayout`
/// (`UI-4`, `UI-5`, `UI-6`).
#[component]
fn Shell() -> Element {
    let ctx_sig = use_context::<Signal<Option<AppContext>>>();
    let app = ctx_sig().expect("shell requires an unlocked context");
    use_context_provider(|| app.clone());
    let onboarding = use_context::<Signal<bool>>();

    data::subscribe();
    let theme = app
        .store
        .app_settings()
        .map(|s| s.color_theme)
        .unwrap_or_default();
    // Immersive surfaces (play, chat) and the first-run story hide all chrome.
    let immersive = SCREEN.read().is_immersive() || onboarding();
    let show_chrome = !immersive;
    let content_class = if show_chrome {
        "text-primary-text flex-grow w-full min-w-0 scrollbar-premium page-transition pb-16 md:pb-0"
    } else {
        "text-primary-text flex-grow w-full min-w-0 scrollbar-premium page-transition"
    };

    rsx! {
        main {
            class: "{theme_class(theme)} relative flex flex-col h-screen overflow-hidden bg-premium-pattern text-primary-text",
            style: "overscroll-behavior-y: contain;",

                if show_chrome {
                    div { class: "pointer-events-none absolute inset-x-0 top-0 z-0 h-28 bg-gradient-to-b from-primary/24 via-primary/10 to-transparent" }
                    div { class: "pointer-events-none absolute left-[-5rem] top-[-3rem] z-0 h-36 w-36 rounded-full bg-primary/16 blur-3xl" }
                    TitleBar {}
                }

                div { class: "flex flex-grow max-w-screen overflow-hidden border-none",
                    if show_chrome {
                        DesktopSidebar {}
                    }
                    div { class: "{content_class}",
                        if onboarding() {
                            {screens::onboarding::render_first_run()}
                        } else {
                            {screens::render_active()}
                        }
                    }
                }

                if show_chrome {
                    MobileBottomNav {}
                }
        }
    }
}

/// Fixed title bar with the app icon, wordmark, and profile button (ported from
/// Soulfire-OG `TitleBar`).
#[component]
fn TitleBar() -> Element {
    rsx! {
        div {
            class: "titlebar-gradient fixed inset-x-0 top-0 z-30 flex h-16 min-h-16 max-h-16 items-center justify-between px-4 text-primary-text sm:px-6",

            button {
                class: "flex items-center gap-3 p-1 rounded-full cursor-pointer transition-all duration-200 hover:bg-white/10 active:bg-white/5",
                onclick: move |_| navigate(Screen::WorldsHome),
                div {
                    class: "flex items-center justify-center w-11 h-11 rounded-full border-2 overflow-hidden",
                    style: "border-color: var(--color-primary-darker); padding: 2px;",
                    img { src: APP_ICON, class: "w-full h-full object-cover rounded-full", alt: "App Icon" }
                }
            }

            div {
                class: "flex items-baseline gap-2 sm:gap-3",
                style: "text-shadow: 0 2px 8px rgba(0,0,0,0.3), 0 4px 16px rgba(0,0,0,0.2);",
                h1 { class: "soulfire-wordmark text-3xl leading-none sm:text-4xl", "Soulfire" }
            }

            button {
                class: "rounded-full w-12 h-12 cursor-pointer transition-all duration-200 hover:scale-105 active:scale-95 border-2 overflow-hidden",
                style: "border-color: var(--color-primary-dark); padding: 2px;",
                onclick: move |_| navigate(Screen::Profile),
                img { src: MISSING_PROFILE, alt: "Profile", class: "w-full h-full object-cover rounded-full" }
            }
        }
    }
}

/// Desktop left sidebar (ported from Soulfire-OG `DesktopSidebar`).
#[component]
fn DesktopSidebar() -> Element {
    let current = SCREEN.read().destination();
    rsx! {
        aside {
            class: "relative hidden md:flex w-64 shrink-0 overflow-hidden border-r border-white/10 bg-[linear-gradient(180deg,rgba(8,8,12,0.98)_0%,rgba(14,12,24,0.98)_58%,rgba(10,9,18,0.98)_100%)] shadow-[inset_-1px_0_0_rgba(255,255,255,0.04)] backdrop-blur-xl",

            div { class: "pointer-events-none absolute inset-0",
                div { class: "absolute inset-x-0 top-0 h-40 bg-gradient-to-b from-primary/20 via-primary/6 to-transparent" }
                div { class: "absolute left-[-3.5rem] top-16 h-36 w-36 rounded-full bg-primary/14 blur-3xl" }
                div { class: "absolute right-[-2rem] bottom-10 h-32 w-32 rounded-full bg-primary/8 blur-3xl" }
                div { class: "absolute inset-y-0 right-0 w-px bg-gradient-to-b from-white/0 via-white/12 to-white/0" }
            }

            div { class: "relative z-10 flex h-full w-full flex-col px-4 pb-6 pt-22",
                nav { class: "flex flex-col gap-2",
                    DesktopSidebarLink {
                        dest: Destination::Worlds,
                        label: "Worlds",
                        description: "Browse worlds and manage adventures",
                        active: current == Destination::Worlds,
                        icon: rsx! { lucide_dioxus::Globe { size: 20 } },
                    }
                    DesktopSidebarLink {
                        dest: Destination::Characters,
                        label: "Characters",
                        description: "Open character chats and creation",
                        active: current == Destination::Characters,
                        icon: rsx! { lucide_dioxus::Drama { size: 20 } },
                    }
                    DesktopSidebarLink {
                        dest: Destination::Settings,
                        label: "Settings",
                        description: "Theme, chat defaults, and adventure defaults",
                        active: current == Destination::Settings,
                        icon: rsx! { lucide_dioxus::Settings { size: 20 } },
                    }
                }

                div { class: "mt-auto rounded-2xl border border-white/10 bg-white/[0.06] p-4 shadow-lg shadow-black/20 backdrop-blur-md",
                    p { class: "text-sm font-medium text-primary-text", "Profile" }
                    p { class: "mt-1 text-sm text-secondary-text",
                        "Use the avatar in the top bar for your profile and account actions."
                    }
                }
            }
        }
    }
}

/// Mobile bottom navigation (ported from Soulfire-OG `MobileBottomNav`).
#[component]
fn MobileBottomNav() -> Element {
    let current = SCREEN.read().destination();
    rsx! {
        nav {
            class: "md:hidden fixed bottom-0 left-0 right-0 z-40 border-t border-white/10 bg-background/95 backdrop-blur-xl",
            div { class: "mx-auto flex h-16 max-w-xl items-center justify-around px-2",
                BottomNavLink {
                    dest: Destination::Worlds,
                    label: "Worlds",
                    active: current == Destination::Worlds,
                    icon: rsx! { lucide_dioxus::Globe { size: 22 } },
                }
                BottomNavLink {
                    dest: Destination::Characters,
                    label: "Characters",
                    active: current == Destination::Characters,
                    icon: rsx! { lucide_dioxus::Drama { size: 22 } },
                }
                BottomNavLink {
                    dest: Destination::Settings,
                    label: "Settings",
                    active: current == Destination::Settings,
                    icon: rsx! { lucide_dioxus::Settings { size: 22 } },
                }
            }
        }
    }
}

#[component]
fn BottomNavLink(dest: Destination, label: &'static str, active: bool, icon: Element) -> Element {
    let class = if active {
        "flex flex-1 flex-col items-center justify-center gap-1 py-2 text-primary-light"
    } else {
        "flex flex-1 flex-col items-center justify-center gap-1 py-2 text-secondary-text"
    };
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| navigate(dest_to_screen(dest)),
            div { class: "flex h-6 w-6 items-center justify-center", {icon} }
            span { class: "text-[11px] font-medium", "{label}" }
        }
    }
}

#[component]
fn DesktopSidebarLink(
    dest: Destination,
    label: &'static str,
    description: &'static str,
    active: bool,
    icon: Element,
) -> Element {
    let class = if active {
        "group flex w-full text-left items-start gap-3 rounded-2xl border border-white/10 bg-white/10 px-4 py-3 text-primary-light shadow-sm"
    } else {
        "group flex w-full text-left items-start gap-3 rounded-2xl border border-transparent px-4 py-3 text-secondary-text transition-colors hover:border-white/10 hover:bg-white/5 hover:text-primary-text"
    };
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| navigate(dest_to_screen(dest)),
            div { class: "mt-0.5 flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-black/15 text-current", {icon} }
            div { class: "min-w-0",
                p { class: "font-medium text-sm", "{label}" }
                p { class: "mt-1 text-xs leading-5 text-secondary-text group-hover:text-secondary-text", "{description}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test (TEST-6 supplement): the app mounts and renders its launch/lock
    /// screen without panicking, with the brand and an unlock affordance present.
    #[test]
    fn app_renders_lock_screen() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("Soulfire"), "brand wordmark present");
        let lower = html.to_lowercase();
        assert!(
            lower.contains("master password") || lower.contains("unlock"),
            "an unlock/setup affordance is present"
        );
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
