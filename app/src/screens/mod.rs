//! Screen components and the active-screen dispatcher.

pub mod builders;
pub mod chat;
pub mod editors;
pub mod onboarding;
pub mod play;
pub mod prompt_viewer;
pub mod settings;
pub mod worlds;

use dioxus::prelude::*;

use crate::nav::{SCREEN, Screen};

/// Render the currently-active screen.
pub fn render_active() -> Element {
    match SCREEN() {
        Screen::WorldsHome => rsx! { worlds::WorldsHome {} },
        Screen::Play(id) => rsx! { play::Play { adventure_id: id } },
        Screen::Characters => rsx! { chat::Characters {} },
        Screen::Chat(id) => rsx! { chat::Chat { character_id: id } },
        Screen::CharacterEditor(id) => rsx! { editors::CharacterEditor { id } },
        Screen::WorldEditor(id) => rsx! { editors::WorldEditor { id } },
        Screen::Settings => rsx! { settings::Settings {} },
        Screen::Profile => rsx! { settings::Profile {} },
        Screen::Stats => rsx! { settings::Stats {} },
        Screen::PromptViewer(id) => rsx! { prompt_viewer::PromptViewer { character_id: id } },
        Screen::AdventurePrompt(id) => {
            rsx! { prompt_viewer::AdventurePromptViewer { adventure_id: id } }
        }
        Screen::CharacterBuilder(id) => rsx! { builders::CharacterBuilder { character_id: id } },
        Screen::WorldBuilder(id) => rsx! { builders::WorldBuilder { blueprint_id: id } },
    }
}

/// A standard page: OG's `PageLayout` header + `ExpandedScrollableLayout` body
/// (ported from Soulfire-OG `components/layout.rs`).
#[component]
pub fn Page(
    title: String,
    children: Element,
    #[props(default)] back: Option<EventHandler<()>>,
    #[props(default)] right: Option<Element>,
    #[props(default)] tabs: Option<Element>,
) -> Element {
    rsx! {
        PageLayout { title, back, right, tabs,
            ExpandedScrollableLayout { {children} }
        }
    }
}

/// OG `PageLayout`: a fixed-titlebar-aware header with title, optional back
/// button, tabs, and right-side content over a premium divider.
#[component]
pub fn PageLayout(
    title: String,
    #[props(default)] back: Option<EventHandler<()>>,
    children: Element,
    #[props(default)] right: Option<Element>,
    #[props(default)] tabs: Option<Element>,
) -> Element {
    rsx! {
        div { class: "flex h-full flex-col pb-3 pt-20",
            div { class: "flex flex-col px-3 sm:px-6 pb-3",
                div { class: "flex flex-wrap items-center space-x-3 sm:space-x-4 justify-between",
                    div { class: "flex items-center space-x-3 sm:space-x-4 min-w-0",
                        if let Some(back) = back {
                            button {
                                class: "flex-shrink-0 transition-transform duration-150 hover:scale-105 active:scale-95",
                                onclick: move |_| back.call(()),
                                BackButton {}
                            }
                        }
                        div { class: "flex items-center gap-2 h-12 sm:h-14 min-w-0",
                            h1 { class: "text-xl sm:text-2xl md:text-2xl font-semibold text-primary dark:text-primary-light truncate tracking-tight",
                                "{title}"
                            }
                        }
                    }
                    if let Some(tabs) = tabs {
                        div { class: "flex items-center justify-center gap-2 order-last w-full sm:order-none sm:w-auto sm:flex-1 pt-2 sm:pt-0",
                            {tabs}
                        }
                    }
                    div { class: "flex items-center space-x-2 sm:space-x-3 flex-shrink-0",
                        if let Some(right) = right {
                            div { class: "flex items-center", {right} }
                        }
                    }
                }
            }
            div { class: "divider-premium mx-3 sm:mx-6" }
            {children}
        }
    }
}

/// OG `ExpandedScrollableLayout`: a centered, scrollable content column.
#[component]
pub fn ExpandedScrollableLayout(children: Element, #[props(default)] wide: bool) -> Element {
    let inner_class = if wide {
        "w-full max-w-5xl mx-auto space-y-4 sm:space-y-5"
    } else {
        "w-full max-w-3xl mx-auto space-y-4 sm:space-y-5"
    };
    rsx! {
        div { class: "flex flex-col h-full overflow-hidden", style: "overscroll-behavior-y: contain;",
            div { class: "flex flex-col flex-grow max-w-screen overflow-y-auto p-3 sm:p-6 scrollbar-premium",
                div { class: inner_class, {children} }
            }
        }
    }
}

/// OG `FillSectionLayout`: content fills the viewport; children scroll their own
/// overflow (used by editors with a single large textarea).
#[component]
pub fn FillSectionLayout(children: Element, #[props(default)] wide: bool) -> Element {
    let inner_class = if wide {
        "w-full max-w-5xl mx-auto flex flex-col flex-1 min-h-0"
    } else {
        "w-full max-w-3xl mx-auto flex flex-col flex-1 min-h-0"
    };
    rsx! {
        div { class: "flex flex-col h-full overflow-hidden", style: "overscroll-behavior-y: contain;",
            div { class: "flex flex-col flex-1 min-h-0 max-w-screen p-3 sm:p-6",
                div { class: inner_class, {children} }
            }
        }
    }
}

/// OG `BottomPanel`: a sticky bottom action/composer bar.
#[component]
pub fn BottomPanel(children: Element) -> Element {
    rsx! {
        div { class: "flex flex-row justify-center px-3 sm:px-6 py-4 sm:py-5 border-t border-border/50 dark:border-white/5 bg-surface/50 dark:bg-surface/30 backdrop-blur-sm",
            div { class: "w-full max-w-3xl", {children} }
        }
    }
}

/// OG back-chevron button glyph (from `components/buttons.rs::BackSvg`).
#[component]
pub fn BackButton() -> Element {
    rsx! {
        svg {
            class: "w-10 h-10 p-2 text-gray-200 hover:text-primary cursor-pointer bg-gray-700 rounded-full",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            path { d: "M15 18l-6-6 6-6" }
        }
    }
}

/// A consistent empty-state block (`UI-8`, `UI-22`).
#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div { class: "text-center text-secondary-text py-16 text-lg", "{message}" }
    }
}
