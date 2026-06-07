//! Screen components and the active-screen dispatcher.

pub mod chat;
pub mod editors;
pub mod play;
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
    }
}

/// A standard-page wrapper: constrained-width content with a heading (`UI-4`).
#[component]
pub fn Page(title: String, children: Element) -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto w-full px-4 py-6 pb-24 md:pb-6",
            h1 { class: "text-2xl font-serif text-primary-text mb-5", "{title}" }
            {children}
        }
    }
}

/// A consistent empty-state block (`UI-8`, `UI-22`).
#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div { class: "text-center text-secondary-text py-16 font-serif text-lg", "{message}" }
    }
}
