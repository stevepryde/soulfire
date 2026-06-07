//! Theme application (`UI-1`, `UI-2`). The app is dark-only; the accent color is
//! applied as a root CSS class that selects the per-theme `--primary-*` ramp.

use dioxus::prelude::*;
use lib_soulfire::settings::ColorTheme;

static CURRENT_THEME: GlobalSignal<ColorTheme> = Signal::global(ColorTheme::default);

/// The root element classes for a theme: always `dark`, plus the accent class
/// (Purple is the default ramp and needs no extra class).
pub fn theme_class(theme: ColorTheme) -> &'static str {
    match theme {
        ColorTheme::Purple => "dark",
        ColorTheme::Blue => "dark theme-blue",
        ColorTheme::Green => "dark theme-green",
        ColorTheme::Red => "dark theme-red",
        ColorTheme::Orange => "dark theme-orange",
        ColorTheme::Teal => "dark theme-teal",
        ColorTheme::Grey => "dark theme-grey",
    }
}

pub fn current_theme() -> ColorTheme {
    CURRENT_THEME()
}

pub fn set_theme(theme: ColorTheme) {
    *CURRENT_THEME.write() = theme;
}
