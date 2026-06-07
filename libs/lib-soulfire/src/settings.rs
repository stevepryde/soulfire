//! App settings (one row, `DATA-18`): the accent color theme and the
//! content/prompt toggles owned by `PROMPT`.

use serde::{Deserialize, Serialize};

/// The user-tunable accent color (`UI-1`). Dark theme only; this is the single
/// tunable theme dimension. Seven options, Purple default.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Default,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumIter,
    strum::EnumString,
    strum::Display,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ColorTheme {
    #[default]
    Purple,
    Blue,
    Green,
    Red,
    Orange,
    Teal,
    Grey,
}

impl ColorTheme {
    /// All seven options in display order (`UI-20` appearance selector).
    pub const ALL: [ColorTheme; 7] = [
        ColorTheme::Purple,
        ColorTheme::Blue,
        ColorTheme::Green,
        ColorTheme::Red,
        ColorTheme::Orange,
        ColorTheme::Teal,
        ColorTheme::Grey,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            ColorTheme::Purple => "Purple",
            ColorTheme::Blue => "Blue",
            ColorTheme::Green => "Green",
            ColorTheme::Red => "Red",
            ColorTheme::Orange => "Orange",
            ColorTheme::Teal => "Teal",
            ColorTheme::Grey => "Grey",
        }
    }

    /// A representative accent hex for the color-theme selector preview swatch
    /// (`UI-1`). The full per-theme ramp lives in the UI stylesheet.
    pub fn preview_hex(&self) -> &'static str {
        match self {
            ColorTheme::Purple => "#8b5cf6",
            ColorTheme::Blue => "#3b82f6",
            ColorTheme::Green => "#22c55e",
            ColorTheme::Red => "#ef4444",
            ColorTheme::Orange => "#f97316",
            ColorTheme::Teal => "#14b8a6",
            ColorTheme::Grey => "#6b7280",
        }
    }
}

/// A user-facing content toggle that gates a clearly-delimited sub-section of a
/// locked prompt block (`PROMPT-6`, `PROMPT-7`). The set is fixed and enumerable.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum ContentToggle {
    /// Controls whether the mature-roleplay stance is included (`PROMPT-6`,
    /// `PROMPT-8`). Defaults off.
    AdultContent,
}

/// The content/prompt toggle state (`DATA-18`, `PROMPT-6`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContentToggles {
    /// Adult-content stance enabled (`PROMPT-8`: defaults off).
    #[serde(default)]
    pub adult_content: bool,
}

impl ContentToggles {
    /// Whether a given toggle is currently enabled.
    pub fn is_enabled(&self, toggle: ContentToggle) -> bool {
        match toggle {
            ContentToggle::AdultContent => self.adult_content,
        }
    }

    /// Set a toggle's state.
    pub fn set(&mut self, toggle: ContentToggle, enabled: bool) {
        match toggle {
            ContentToggle::AdultContent => self.adult_content = enabled,
        }
    }

    /// Enumerate every toggle and its current state (`PROMPT-7`: fixed/enumerable).
    pub fn all(&self) -> [(ContentToggle, bool); 1] {
        [(ContentToggle::AdultContent, self.adult_content)]
    }
}

/// App settings (one row, `DATA-18`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, bon::Builder)]
pub struct AppSettings {
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    /// The active accent color theme (default Purple, `UI-1`).
    #[builder(default)]
    #[serde(default)]
    pub color_theme: ColorTheme,
    /// Content/prompt toggles (adult content defaults off, `PROMPT-8`).
    #[builder(default)]
    #[serde(default)]
    pub content_toggles: ContentToggles,
}

fn one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec() {
        // DATA-18 / PROMPT-8: default theme Purple, adult content off.
        let s = AppSettings::default();
        assert_eq!(s.color_theme, ColorTheme::Purple);
        assert!(!s.content_toggles.adult_content);
    }

    #[test]
    fn color_theme_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ColorTheme::Teal).unwrap(),
            "\"teal\""
        );
        assert_eq!(ColorTheme::ALL.len(), 7);
    }

    #[test]
    fn content_toggle_enumerates_and_sets() {
        let mut t = ContentToggles::default();
        assert!(!t.is_enabled(ContentToggle::AdultContent));
        t.set(ContentToggle::AdultContent, true);
        assert!(t.is_enabled(ContentToggle::AdultContent));
        assert_eq!(t.all().len(), 1);
    }
}
