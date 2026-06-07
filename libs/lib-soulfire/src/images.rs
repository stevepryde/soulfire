//! Avatar/cover emoji-and-illustration enumerations (`DATA-20`).
//!
//! Reproduced verbatim from Soulfire-OG: characters choose from 20 emoji avatars
//! plus 8 bundled illustrated characters (Lyra, Solas, Nova, Virel, Iris, Nikhil,
//! Kiran, Thorne); worlds choose from ~80 emoji covers. The variant set, their
//! serialized strings, and their emoji/asset mappings are contract values.

use serde::{Deserialize, Serialize};

/// A character avatar selection: an emoji or a bundled illustrated portrait.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum CharacterImage {
    // Emoji variants
    #[default]
    EmojiButterfly,
    EmojiRobot,
    EmojiAlien,
    EmojiCat,
    EmojiDog,
    EmojiBear,
    EmojiPanda,
    EmojiLion,
    EmojiTiger,
    EmojiFox,
    EmojiWolf,
    EmojiMonkey,
    EmojiUnicorn,
    EmojiDragon,
    EmojiOwl,
    EmojiPenguin,
    EmojiKoala,
    EmojiFrog,
    EmojiOctopus,
    EmojiBee,
    // Illustrated character images
    Lyra,
    Solas,
    Nova,
    Virel,
    Iris,
    Nikhil,
    Kiran,
    Thorne,
}

impl CharacterImage {
    /// The bundled illustration filename, or `""` for emoji variants.
    pub fn image_filename(&self) -> &'static str {
        match self {
            CharacterImage::Lyra => "lyra200.png",
            CharacterImage::Nova => "nova200.png",
            CharacterImage::Solas => "solas200.png",
            CharacterImage::Virel => "virel200.png",
            CharacterImage::Iris => "iris200.png",
            CharacterImage::Nikhil => "nikhil200.png",
            CharacterImage::Kiran => "kiran200.png",
            CharacterImage::Thorne => "thorne200.png",
            _ => "",
        }
    }

    /// The emoji glyph for emoji variants; `None` for illustrated portraits.
    pub fn emoji(&self) -> Option<&'static str> {
        Some(match self {
            CharacterImage::EmojiRobot => "🤖",
            CharacterImage::EmojiAlien => "👽",
            CharacterImage::EmojiCat => "🐱",
            CharacterImage::EmojiDog => "🐶",
            CharacterImage::EmojiBear => "🐻",
            CharacterImage::EmojiPanda => "🐼",
            CharacterImage::EmojiLion => "🦁",
            CharacterImage::EmojiTiger => "🐯",
            CharacterImage::EmojiFox => "🦊",
            CharacterImage::EmojiWolf => "🐺",
            CharacterImage::EmojiMonkey => "🐵",
            CharacterImage::EmojiUnicorn => "🦄",
            CharacterImage::EmojiDragon => "🐉",
            CharacterImage::EmojiOwl => "🦉",
            CharacterImage::EmojiPenguin => "🐧",
            CharacterImage::EmojiKoala => "🐨",
            CharacterImage::EmojiFrog => "🐸",
            CharacterImage::EmojiOctopus => "🐙",
            CharacterImage::EmojiButterfly => "🦋",
            CharacterImage::EmojiBee => "🐝",
            _ => return None,
        })
    }

    pub fn is_emoji(&self) -> bool {
        self.emoji().is_some()
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CharacterImage::Lyra => "Lyra",
            CharacterImage::Solas => "Solas",
            CharacterImage::Nova => "Nova",
            CharacterImage::Virel => "Virel",
            CharacterImage::Iris => "Iris",
            CharacterImage::Nikhil => "Nikhil",
            CharacterImage::Kiran => "Kiran",
            CharacterImage::Thorne => "Thorne",
            CharacterImage::EmojiRobot => "Robot",
            CharacterImage::EmojiAlien => "Alien",
            CharacterImage::EmojiCat => "Cat",
            CharacterImage::EmojiDog => "Dog",
            CharacterImage::EmojiBear => "Bear",
            CharacterImage::EmojiPanda => "Panda",
            CharacterImage::EmojiLion => "Lion",
            CharacterImage::EmojiTiger => "Tiger",
            CharacterImage::EmojiFox => "Fox",
            CharacterImage::EmojiWolf => "Wolf",
            CharacterImage::EmojiMonkey => "Monkey",
            CharacterImage::EmojiUnicorn => "Unicorn",
            CharacterImage::EmojiDragon => "Dragon",
            CharacterImage::EmojiOwl => "Owl",
            CharacterImage::EmojiPenguin => "Penguin",
            CharacterImage::EmojiKoala => "Koala",
            CharacterImage::EmojiFrog => "Frog",
            CharacterImage::EmojiOctopus => "Octopus",
            CharacterImage::EmojiButterfly => "Butterfly",
            CharacterImage::EmojiBee => "Bee",
        }
    }
}

/// A world cover selection (emoji only).
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    strum::EnumString,
    strum::Display,
    strum::EnumIter,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum WorldImage {
    #[default]
    EmojiGlobe,
    EmojiCastle,
    EmojiMountain,
    EmojiIsland,
    EmojiForest,
    EmojiDesert,
    EmojiVolcano,
    EmojiCrystal,
    EmojiStars,
    EmojiMoon,
    EmojiSun,
    EmojiRainbow,
    EmojiFire,
    EmojiWater,
    EmojiLeaf,
    EmojiSnowflake,
    EmojiLightning,
    EmojiTornado,
    EmojiDragon,
    EmojiSword,
    EmojiShield,
    EmojiScroll,
    EmojiKey,
    EmojiTreasure,
    EmojiCompass,
    EmojiShip,
    EmojiAirplane,
    EmojiRocket,
    EmojiTrain,
    EmojiCar,
    EmojiHorse,
    EmojiBike,
    EmojiBook,
    EmojiMagicWand,
    EmojiCrystalBall,
    EmojiCrown,
    EmojiRing,
    EmojiHourglass,
    EmojiAnchor,
    EmojiTelescope,
    EmojiMicroscope,
    EmojiRobot,
    EmojiAlien,
    EmojiGhost,
    EmojiSkull,
    EmojiUnicorn,
    EmojiWolf,
    EmojiLion,
    EmojiEagle,
    EmojiSnake,
    EmojiOctopus,
    EmojiButterfly,
    EmojiMushroom,
    EmojiCactus,
    EmojiRose,
    EmojiTent,
    EmojiBridge,
    EmojiLighthouse,
    EmojiFactory,
    EmojiChurch,
    EmojiStatue,
    EmojiFountain,
    EmojiCloud,
    EmojiFog,
    EmojiComet,
    EmojiMeteor,
    EmojiGalaxy,
    EmojiPotion,
    EmojiDagger,
    EmojiBow,
    EmojiAxe,
    EmojiHammer,
    EmojiWrench,
    EmojiGear,
    EmojiLock,
    EmojiChain,
    EmojiBell,
    EmojiDrum,
    EmojiGuitar,
}

impl WorldImage {
    pub fn emoji(&self) -> &'static str {
        match self {
            WorldImage::EmojiGlobe => "🌍",
            WorldImage::EmojiCastle => "🏰",
            WorldImage::EmojiMountain => "⛰️",
            WorldImage::EmojiIsland => "🏝️",
            WorldImage::EmojiForest => "🌲",
            WorldImage::EmojiDesert => "🏜️",
            WorldImage::EmojiVolcano => "🌋",
            WorldImage::EmojiCrystal => "💎",
            WorldImage::EmojiStars => "✨",
            WorldImage::EmojiMoon => "🌙",
            WorldImage::EmojiSun => "☀️",
            WorldImage::EmojiRainbow => "🌈",
            WorldImage::EmojiFire => "🔥",
            WorldImage::EmojiWater => "💧",
            WorldImage::EmojiLeaf => "🍃",
            WorldImage::EmojiSnowflake => "❄️",
            WorldImage::EmojiLightning => "⚡",
            WorldImage::EmojiTornado => "🌪️",
            WorldImage::EmojiDragon => "🐉",
            WorldImage::EmojiSword => "⚔️",
            WorldImage::EmojiShield => "🛡️",
            WorldImage::EmojiScroll => "📜",
            WorldImage::EmojiKey => "🔑",
            WorldImage::EmojiTreasure => "💰",
            WorldImage::EmojiCompass => "🧭",
            WorldImage::EmojiShip => "🚢",
            WorldImage::EmojiAirplane => "✈️",
            WorldImage::EmojiRocket => "🚀",
            WorldImage::EmojiTrain => "🚂",
            WorldImage::EmojiCar => "🚗",
            WorldImage::EmojiHorse => "🐴",
            WorldImage::EmojiBike => "🚴",
            WorldImage::EmojiBook => "📖",
            WorldImage::EmojiMagicWand => "🪄",
            WorldImage::EmojiCrystalBall => "🔮",
            WorldImage::EmojiCrown => "👑",
            WorldImage::EmojiRing => "💍",
            WorldImage::EmojiHourglass => "⏳",
            WorldImage::EmojiAnchor => "⚓",
            WorldImage::EmojiTelescope => "🔭",
            WorldImage::EmojiMicroscope => "🔬",
            WorldImage::EmojiRobot => "🤖",
            WorldImage::EmojiAlien => "👽",
            WorldImage::EmojiGhost => "👻",
            WorldImage::EmojiSkull => "💀",
            WorldImage::EmojiUnicorn => "🦄",
            WorldImage::EmojiWolf => "🐺",
            WorldImage::EmojiLion => "🦁",
            WorldImage::EmojiEagle => "🦅",
            WorldImage::EmojiSnake => "🐍",
            WorldImage::EmojiOctopus => "🐙",
            WorldImage::EmojiButterfly => "🦋",
            WorldImage::EmojiMushroom => "🍄",
            WorldImage::EmojiCactus => "🌵",
            WorldImage::EmojiRose => "🌹",
            WorldImage::EmojiTent => "⛺",
            WorldImage::EmojiBridge => "🌉",
            WorldImage::EmojiLighthouse => "🗼",
            WorldImage::EmojiFactory => "🏭",
            WorldImage::EmojiChurch => "⛪",
            WorldImage::EmojiStatue => "🗿",
            WorldImage::EmojiFountain => "⛲",
            WorldImage::EmojiCloud => "☁️",
            WorldImage::EmojiFog => "🌫️",
            WorldImage::EmojiComet => "☄️",
            WorldImage::EmojiMeteor => "💫",
            WorldImage::EmojiGalaxy => "🌌",
            WorldImage::EmojiPotion => "🧪",
            WorldImage::EmojiDagger => "🗡️",
            WorldImage::EmojiBow => "🏹",
            WorldImage::EmojiAxe => "🪓",
            WorldImage::EmojiHammer => "🔨",
            WorldImage::EmojiWrench => "🔧",
            WorldImage::EmojiGear => "⚙️",
            WorldImage::EmojiLock => "🔒",
            WorldImage::EmojiChain => "⛓️",
            WorldImage::EmojiBell => "🔔",
            WorldImage::EmojiDrum => "🥁",
            WorldImage::EmojiGuitar => "🎸",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WorldImage::EmojiGlobe => "Globe",
            WorldImage::EmojiCastle => "Castle",
            WorldImage::EmojiMountain => "Mountain",
            WorldImage::EmojiIsland => "Island",
            WorldImage::EmojiForest => "Forest",
            WorldImage::EmojiDesert => "Desert",
            WorldImage::EmojiVolcano => "Volcano",
            WorldImage::EmojiCrystal => "Crystal",
            WorldImage::EmojiStars => "Stars",
            WorldImage::EmojiMoon => "Moon",
            WorldImage::EmojiSun => "Sun",
            WorldImage::EmojiRainbow => "Rainbow",
            WorldImage::EmojiFire => "Fire",
            WorldImage::EmojiWater => "Water",
            WorldImage::EmojiLeaf => "Leaf",
            WorldImage::EmojiSnowflake => "Snowflake",
            WorldImage::EmojiLightning => "Lightning",
            WorldImage::EmojiTornado => "Tornado",
            WorldImage::EmojiDragon => "Dragon",
            WorldImage::EmojiSword => "Sword",
            WorldImage::EmojiShield => "Shield",
            WorldImage::EmojiScroll => "Scroll",
            WorldImage::EmojiKey => "Key",
            WorldImage::EmojiTreasure => "Treasure",
            WorldImage::EmojiCompass => "Compass",
            WorldImage::EmojiShip => "Ship",
            WorldImage::EmojiAirplane => "Airplane",
            WorldImage::EmojiRocket => "Rocket",
            WorldImage::EmojiTrain => "Train",
            WorldImage::EmojiCar => "Car",
            WorldImage::EmojiHorse => "Horse",
            WorldImage::EmojiBike => "Bike",
            WorldImage::EmojiBook => "Book",
            WorldImage::EmojiMagicWand => "Magic Wand",
            WorldImage::EmojiCrystalBall => "Crystal Ball",
            WorldImage::EmojiCrown => "Crown",
            WorldImage::EmojiRing => "Ring",
            WorldImage::EmojiHourglass => "Hourglass",
            WorldImage::EmojiAnchor => "Anchor",
            WorldImage::EmojiTelescope => "Telescope",
            WorldImage::EmojiMicroscope => "Microscope",
            WorldImage::EmojiRobot => "Robot",
            WorldImage::EmojiAlien => "Alien",
            WorldImage::EmojiGhost => "Ghost",
            WorldImage::EmojiSkull => "Skull",
            WorldImage::EmojiUnicorn => "Unicorn",
            WorldImage::EmojiWolf => "Wolf",
            WorldImage::EmojiLion => "Lion",
            WorldImage::EmojiEagle => "Eagle",
            WorldImage::EmojiSnake => "Snake",
            WorldImage::EmojiOctopus => "Octopus",
            WorldImage::EmojiButterfly => "Butterfly",
            WorldImage::EmojiMushroom => "Mushroom",
            WorldImage::EmojiCactus => "Cactus",
            WorldImage::EmojiRose => "Rose",
            WorldImage::EmojiTent => "Tent",
            WorldImage::EmojiBridge => "Bridge",
            WorldImage::EmojiLighthouse => "Lighthouse",
            WorldImage::EmojiFactory => "Factory",
            WorldImage::EmojiChurch => "Church",
            WorldImage::EmojiStatue => "Statue",
            WorldImage::EmojiFountain => "Fountain",
            WorldImage::EmojiCloud => "Cloud",
            WorldImage::EmojiFog => "Fog",
            WorldImage::EmojiComet => "Comet",
            WorldImage::EmojiMeteor => "Meteor",
            WorldImage::EmojiGalaxy => "Galaxy",
            WorldImage::EmojiPotion => "Potion",
            WorldImage::EmojiDagger => "Dagger",
            WorldImage::EmojiBow => "Bow",
            WorldImage::EmojiAxe => "Axe",
            WorldImage::EmojiHammer => "Hammer",
            WorldImage::EmojiWrench => "Wrench",
            WorldImage::EmojiGear => "Gear",
            WorldImage::EmojiLock => "Lock",
            WorldImage::EmojiChain => "Chain",
            WorldImage::EmojiBell => "Bell",
            WorldImage::EmojiDrum => "Drum",
            WorldImage::EmojiGuitar => "Guitar",
        }
    }
}

/// A reference to a stored (generated or uploaded) image whose bytes live in the
/// encrypted store, keyed by the owning entity (`IMG-4`, `SEC-3`). The `version`
/// is a cache-bust value bumped on each regeneration so the UI re-renders
/// (`IMG-3`/`AC-IMG-b`). Presence of this reference means a stored image exists and
/// takes precedence over the emoji selection (`IMG-8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct StoredImageRef {
    #[builder(default = 1)]
    #[serde(default = "default_image_version")]
    pub version: u32,
}

fn default_image_version() -> u32 {
    1
}

impl StoredImageRef {
    /// A fresh reference at version 1.
    pub fn new() -> Self {
        StoredImageRef { version: 1 }
    }

    /// Return a copy with the cache-bust version advanced (`IMG-3`).
    pub fn bumped(self) -> Self {
        StoredImageRef {
            version: self.version.saturating_add(1),
        }
    }
}

impl Default for StoredImageRef {
    fn default() -> Self {
        StoredImageRef::new()
    }
}

/// Pan/zoom framing for a stored portrait or cover image (`DATA-1`, `DATA-8`,
/// `IMG-7`). Percentages: pan in [-100, 100], zoom default 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
pub struct ImageTransform {
    #[builder(default)]
    #[serde(default)]
    pub pan_x_percent: i16,
    #[builder(default)]
    #[serde(default)]
    pub pan_y_percent: i16,
    #[builder(default = 100)]
    #[serde(default = "default_zoom_percent")]
    pub zoom_percent: u16,
}

fn default_zoom_percent() -> u16 {
    100
}

impl Default for ImageTransform {
    fn default() -> Self {
        ImageTransform {
            pan_x_percent: 0,
            pan_y_percent: 0,
            zoom_percent: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use strum::IntoEnumIterator;

    #[test]
    fn character_image_counts_match_spec() {
        // DATA-20: 20 emoji avatars + 8 illustrated.
        let emoji = CharacterImage::iter().filter(|i| i.is_emoji()).count();
        let illustrated = CharacterImage::iter().filter(|i| !i.is_emoji()).count();
        assert_eq!(emoji, 20);
        assert_eq!(illustrated, 8);
    }

    #[test]
    fn character_image_serializes_snake_case() {
        let json = serde_json::to_string(&CharacterImage::EmojiButterfly).unwrap();
        assert_eq!(json, "\"emoji_butterfly\"");
        assert_eq!(
            CharacterImage::from_str("emoji_butterfly").unwrap(),
            CharacterImage::EmojiButterfly
        );
    }

    #[test]
    fn illustrated_have_filenames_emoji_have_glyphs() {
        assert_eq!(CharacterImage::Lyra.image_filename(), "lyra200.png");
        assert_eq!(CharacterImage::EmojiCat.emoji(), Some("🐱"));
        assert!(CharacterImage::Lyra.emoji().is_none());
    }

    #[test]
    fn world_image_every_variant_has_emoji_and_name() {
        for img in WorldImage::iter() {
            assert!(!img.emoji().is_empty());
            assert!(!img.display_name().is_empty());
        }
    }

    #[test]
    fn world_image_serializes_snake_case() {
        let json = serde_json::to_string(&WorldImage::EmojiMagicWand).unwrap();
        assert_eq!(json, "\"emoji_magic_wand\"");
    }

    #[test]
    fn image_transform_default_is_centered_100() {
        let t = ImageTransform::default();
        assert_eq!(
            (t.pan_x_percent, t.pan_y_percent, t.zoom_percent),
            (0, 0, 100)
        );
    }
}
