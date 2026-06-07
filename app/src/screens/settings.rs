//! Settings, profile, and token statistics screens (`UI-20`, `UI-21`, `STAT`).

use dioxus::prelude::*;

use lib_soulfire::ai_model::{AiModel, AiVendor};
use lib_soulfire::credentials::ProviderCredential;
use lib_soulfire::settings::ColorTheme;

use soulfire_core::stats::{StatsReport, TokenTotals};

use crate::app::current_app;
use crate::data;
use crate::nav::{Screen, navigate};

use super::Page;

/// Settings: appearance, AI, content toggles, adventure defaults, and entries to
/// stats and security (`UI-20`).
#[component]
pub fn Settings() -> Element {
    data::subscribe();
    let app = current_app();
    let settings = app.store.app_settings().unwrap_or_default();
    let player = app.store.player_profile().unwrap_or_default();
    let masked_key = app
        .store
        .credential(AiVendor::OpenAI)
        .ok()
        .flatten()
        .map(|c| c.masked());

    rsx! {
        Page { title: "Settings".to_string(),
            // Appearance — accent color (UI-1).
            Section { title: "Appearance".to_string(),
                p { class: "text-sm text-secondary-text mb-3", "Accent color" }
                div { class: "flex flex-wrap gap-3",
                    for theme in ColorTheme::ALL {
                        {accent_swatch(theme, settings.color_theme)}
                    }
                }
            }

            // AI — API key (SEC-10) + default model (AI-7).
            Section { title: "AI".to_string(),
                ApiKeyField { masked: masked_key }
                div { class: "mt-4",
                    p { class: "text-sm text-secondary-text mb-2", "Default model" }
                    div { class: "flex flex-wrap gap-2",
                        for model in AiModel::ALL {
                            {model_chip(model, app.store.app_profile().ok().and_then(|p| p.default_ai_model))}
                        }
                    }
                }
            }

            // Content toggles (PROMPT-6).
            Section { title: "Content".to_string(),
                ToggleRow {
                    label: "Adult content".to_string(),
                    help: "Allow mature, explicit roleplay in prompts.".to_string(),
                    on: settings.content_toggles.adult_content,
                    ontoggle: move |_| {
                        let mut s = app.store.app_settings().unwrap_or_default();
                        s.content_toggles.adult_content = !s.content_toggles.adult_content;
                        let _ = app.store.save_app_settings(&s);
                        data::touch();
                    },
                }
            }

            // Adventure defaults (DATA-17).
            Section { title: "Adventure Defaults".to_string(),
                AdventureDefaults { player }
            }

            // Entries to stats and security.
            Section { title: "More".to_string(),
                button {
                    class: "block w-full text-left py-2 text-link hover:underline",
                    onclick: move |_| navigate(Screen::Stats),
                    "Token statistics →"
                }
                button {
                    class: "block w-full text-left py-2 text-link hover:underline",
                    onclick: move |_| navigate(Screen::Profile),
                    "Profile →"
                }
            }
        }
    }
}

#[component]
fn AdventureDefaults(player: lib_soulfire::profile::PlayerProfile) -> Element {
    use lib_soulfire::strings::{PlayerAttributes, PlayerName, PromptExtension};
    let app = current_app();
    let mut name = use_signal(|| player.player_name.to_string());
    let mut attrs = use_signal(|| player.player_attributes.to_string());
    let mut ext = use_signal(|| {
        player
            .prompt_extension
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_default()
    });

    rsx! {
        p { class: "text-xs text-secondary-text mb-3", "Used when starting new adventures; affects only adventures started afterward." }
        label { class: "block text-sm text-primary-light mb-1", "Adventurer name" }
        input { class: "input-premium w-full mb-3", value: "{name}", oninput: move |e| name.set(e.value()) }
        label { class: "block text-sm text-primary-light mb-1", "Attributes" }
        textarea { class: "input-premium w-full mb-3", rows: "3", value: "{attrs}", oninput: move |e| attrs.set(e.value()) }
        label { class: "block text-sm text-primary-light mb-1", "Prompt extension (optional)" }
        textarea { class: "input-premium w-full mb-3", rows: "2", value: "{ext}", oninput: move |e| ext.set(e.value()) }
        button {
            class: "crm-primary-button px-5 py-2 rounded-lg text-sm",
            onclick: move |_| {
                let mut p = app.store.player_profile().unwrap_or_default();
                p.player_name = PlayerName::coerce(name().trim());
                p.player_attributes = PlayerAttributes::coerce(attrs().trim());
                let e = ext();
                p.prompt_extension = if e.trim().is_empty() { None } else { Some(PromptExtension::coerce(e.trim())) };
                let _ = app.store.save_player_profile(&p);
                data::touch();
                sp_ui::toast::ToastService::info("Saved.");
            },
            "Save defaults"
        }
    }
}

#[component]
fn Section(title: String, children: Element) -> Element {
    rsx! {
        section { class: "bg-surface border border-border rounded-xl p-5 mb-5",
            h2 { class: "text-lg font-semibold text-primary-text mb-3", "{title}" }
            {children}
        }
    }
}

fn accent_swatch(theme: ColorTheme, current: ColorTheme) -> Element {
    let app = current_app();
    let selected = theme == current;
    let ring = if selected {
        "ring-2 ring-primary border-primary"
    } else {
        "border-border"
    };
    rsx! {
        button {
            class: "flex flex-col items-center gap-2 p-3 rounded-lg border-2 {ring} hover:scale-105 transition-transform",
            onclick: move |_| {
                let mut s = app.store.app_settings().unwrap_or_default();
                s.color_theme = theme;
                let _ = app.store.save_app_settings(&s);
                data::touch();
            },
            div { class: "w-8 h-8 rounded-full border-2 border-white/30", style: "background-color: {theme.preview_hex()};" }
            span { class: "text-xs text-secondary-text", "{theme.display_name()}" }
        }
    }
}

fn model_chip(model: AiModel, current: Option<AiModel>) -> Element {
    let app = current_app();
    let selected = current == Some(model);
    let cls = if selected {
        "bg-primary text-primary-text"
    } else {
        "bg-surface border border-border text-secondary-text"
    };
    rsx! {
        button {
            class: "px-3 py-1.5 rounded-lg text-sm {cls}",
            onclick: move |_| {
                let mut p = app.store.app_profile().unwrap_or_default();
                p.default_ai_model = Some(model);
                let _ = app.store.save_app_profile(&p);
                data::touch();
            },
            "{model.display_name()}"
        }
    }
}

#[component]
fn ApiKeyField(masked: Option<String>) -> Element {
    let app = current_app();
    let mut editing = use_signal(|| false);
    let mut value = use_signal(String::new);

    rsx! {
        p { class: "text-sm text-secondary-text mb-2", "OpenAI API key" }
        if editing() {
            div { class: "flex gap-2",
                input {
                    class: "input-premium flex-1",
                    r#type: "password",
                    placeholder: "sk-…",
                    value: "{value}",
                    oninput: move |e| value.set(e.value()),
                }
                button {
                    class: "crm-primary-button px-4 rounded-lg",
                    onclick: move |_| {
                        let k = value();
                        if !k.trim().is_empty() {
                            let _ = app.store.save_credential(&ProviderCredential::new(AiVendor::OpenAI, k.trim().to_string()));
                        }
                        editing.set(false);
                        value.set(String::new());
                        data::touch();
                    },
                    "Save"
                }
            }
        } else {
            div { class: "flex items-center gap-3",
                span { class: "text-primary-text font-mono",
                    if let Some(m) = masked.clone() { "{m}" } else { "Not set" }
                }
                button {
                    class: "text-link text-sm hover:underline",
                    onclick: move |_| editing.set(true),
                    if masked.is_some() { "Replace" } else { "Add key" }
                }
            }
        }
    }
}

#[component]
fn ToggleRow(label: String, help: String, on: bool, ontoggle: EventHandler<()>) -> Element {
    let track = if on { "bg-primary" } else { "bg-gray-600" };
    let knob = if on { "translate-x-5" } else { "translate-x-0" };
    rsx! {
        div { class: "flex items-center justify-between",
            div {
                p { class: "text-primary-text font-medium", "{label}" }
                p { class: "text-xs text-secondary-text", "{help}" }
            }
            button {
                class: "relative w-11 h-6 rounded-full transition-colors {track}",
                onclick: move |_| ontoggle.call(()),
                div { class: "absolute top-1 left-1 w-4 h-4 bg-white rounded-full transition-transform {knob}" }
            }
        }
    }
}

/// The app profile screen (`UI-21`) with a lock action.
#[component]
pub fn Profile() -> Element {
    data::subscribe();
    let app = current_app();
    let profile = app.store.app_profile().unwrap_or_default();
    let mut ctx = use_context::<Signal<Option<crate::state::AppContext>>>();
    rsx! {
        Page { title: "Profile".to_string(),
            section { class: "bg-surface border border-border rounded-xl p-5 mb-5",
                div { class: "flex items-center gap-4",
                    div { class: "w-16 h-16 rounded-full bg-primary-lighter text-primary flex items-center justify-center text-2xl", "🙂" }
                    div {
                        p { class: "text-lg text-primary-text",
                            if let Some(n) = &profile.name { "{n}" } else { "Unnamed" }
                        }
                        p { class: "text-sm text-secondary-text", "Language: {profile.primary_language}" }
                    }
                }
            }
            button {
                class: "w-full py-3 rounded-lg border border-border text-primary-text hover-highlight",
                onclick: move |_| ctx.set(None), // lock the app (SEC-8)
                "Lock app"
            }
        }
    }
}

/// Token statistics (`STAT-5`).
#[component]
pub fn Stats() -> Element {
    data::subscribe();
    let app = current_app();
    let metrics = app.store.all_metrics().unwrap_or_default();
    let report = StatsReport::from_metrics(&metrics);

    rsx! {
        Page { title: "Token Statistics".to_string(),
            if metrics.is_empty() {
                super::EmptyState { message: "No usage yet. Token counts appear here as you chat and play.".to_string() }
            } else {
                TotalsCard { totals: report.totals }
                section { class: "bg-surface border border-border rounded-xl p-5 mb-5",
                    h2 { class: "text-lg font-semibold mb-3", "By model" }
                    for (model, t) in report.by_model.iter() {
                        BreakdownRow { label: model.display_name().to_string(), totals: *t }
                    }
                }
                section { class: "bg-surface border border-border rounded-xl p-5",
                    h2 { class: "text-lg font-semibold mb-3", "By operation" }
                    for (label, t) in report.by_label.iter() {
                        BreakdownRow { label: label.to_string(), totals: *t }
                    }
                }
                button {
                    class: "mt-5 text-error text-sm hover:underline",
                    onclick: move |_| { let _ = app.store.clear_metrics(); data::touch(); },
                    "Clear usage history"
                }
            }
        }
    }
}

#[component]
fn TotalsCard(totals: TokenTotals) -> Element {
    rsx! {
        section { class: "bg-surface border border-border rounded-xl p-5 mb-5 grid grid-cols-2 sm:grid-cols-4 gap-4",
            Stat { label: "Requests".to_string(), value: totals.requests }
            Stat { label: "Input".to_string(), value: totals.input_tokens }
            Stat { label: "Cached".to_string(), value: totals.cached_input_tokens }
            Stat { label: "Output".to_string(), value: totals.output_tokens }
        }
    }
}

#[component]
fn Stat(label: String, value: u64) -> Element {
    rsx! {
        div {
            p { class: "text-2xl font-semibold text-primary", "{value}" }
            p { class: "text-xs text-secondary-text", "{label}" }
        }
    }
}

#[component]
fn BreakdownRow(label: String, totals: TokenTotals) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 text-sm",
            span { class: "text-primary-text", "{label}" }
            span { class: "text-secondary-text", "{totals.input_tokens} in · {totals.output_tokens} out" }
        }
    }
}
