//! Screen navigation (`UI-6`). A global signal holds the current screen; the
//! sidebar/bottom nav and in-screen actions set it. (A gated single-window app,
//! so a signal-driven screen stack is simpler than a typed router and fully
//! covers the navigation contract.)

use dioxus::prelude::*;

use lib_soulfire::ids::{AdventureId, CharacterId, WorldBlueprintId};

/// The primary navigation destinations (`UI-6`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Worlds,
    Characters,
    Settings,
}

/// The current screen.
#[derive(Clone, PartialEq)]
pub enum Screen {
    /// Worlds home — the default landing for returning users (`UI-8`, `ONB-6`).
    WorldsHome,
    /// The immersive adventure play screen (`UI-10`).
    Play(AdventureId),
    /// The characters & chats list (`UI-17`).
    Characters,
    /// The immersive 1:1 chat screen (`UI-14`).
    Chat(CharacterId),
    /// The character manual editor (`UI-18`); `None` creates a new character.
    CharacterEditor(Option<CharacterId>),
    /// The world blueprint manual editor (`UI-18`); `None` creates a new world.
    WorldEditor(Option<WorldBlueprintId>),
    /// Settings (`UI-20`).
    Settings,
    /// The app profile (`UI-21`).
    Profile,
    /// Token statistics (`STAT`, `UI-20`).
    Stats,
    /// The prompt viewer/editor for a character (`PROMPT-9`).
    PromptViewer(CharacterId),
    /// The conversational character builder (`CHAR-6`).
    CharacterBuilder(CharacterId),
    /// The conversational world builder (`WORLD-20`).
    WorldBuilder(WorldBlueprintId),
}

impl Screen {
    /// The primary nav destination this screen belongs under (for active state).
    pub fn destination(&self) -> Destination {
        match self {
            Screen::WorldsHome
            | Screen::Play(_)
            | Screen::WorldEditor(_)
            | Screen::WorldBuilder(_) => Destination::Worlds,
            Screen::Characters
            | Screen::Chat(_)
            | Screen::CharacterEditor(_)
            | Screen::CharacterBuilder(_)
            | Screen::PromptViewer(_) => Destination::Characters,
            Screen::Settings | Screen::Profile | Screen::Stats => Destination::Settings,
        }
    }

    /// Whether this screen uses the immersive chrome-less surface (`UI-4`).
    pub fn is_immersive(&self) -> bool {
        matches!(self, Screen::Play(_) | Screen::Chat(_))
    }
}

/// The current screen (global).
pub static SCREEN: GlobalSignal<Screen> = Signal::global(|| Screen::WorldsHome);

/// Navigate to a screen.
pub fn navigate(screen: Screen) {
    *SCREEN.write() = screen;
}
