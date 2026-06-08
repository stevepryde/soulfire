//! Singleton profiles: the app profile and the player profile (`DATA-16`,
//! `DATA-17`). Because Soulfire is single-user (`PROD-12`) these are single rows
//! with no `user_id`.

use serde::{Deserialize, Serialize};

use super::ai_model::AiModel;
use super::images::StoredImageRef;
use super::strings::{DisplayName, PlayerAttributes, PlayerName, PromptExtension};

/// The user's preferred language (`DATA-16`). Ported from Soulfire-OG.
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
pub enum Language {
    #[default]
    English,
    Thai,
    Spanish,
    German,
    French,
    Indonesian,
    Italian,
    Japanese,
    #[serde(rename = "brazilian portuguese")]
    #[strum(serialize = "brazilian portuguese")]
    BrazilianPortuguese,
    Portuguese,
}

/// The app profile (one row, `DATA-16`). Soulfire-OG's account/role/email fields
/// are removed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, bon::Builder)]
pub struct AppProfile {
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<DisplayName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<DisplayName>,
    #[builder(default)]
    #[serde(default)]
    pub primary_language: Language,
    /// An avatar image stored in the encrypted store (`IMG`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar: Option<StoredImageRef>,
    /// The profile-wide default model, used when an entity has none (`AI-9`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_ai_model: Option<AiModel>,
}

/// The player profile (one row, `DATA-17`): the default adventurer identity used
/// when starting new adventures. Editing it affects only adventures started
/// afterward (`WORLD-3`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, bon::Builder)]
pub struct PlayerProfile {
    #[builder(default = 1)]
    #[serde(default = "one")]
    pub version: u32,
    #[builder(default)]
    #[serde(default)]
    pub player_name: PlayerName,
    #[builder(default)]
    #[serde(default)]
    pub player_attributes: PlayerAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_extension: Option<PromptExtension>,
}

fn one() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn language_defaults_to_english_and_serializes_snake_case() {
        assert_eq!(Language::default(), Language::English);
        assert_eq!(
            serde_json::to_string(&Language::English).unwrap(),
            "\"english\""
        );
        assert_eq!(
            serde_json::to_string(&Language::BrazilianPortuguese).unwrap(),
            "\"brazilian portuguese\""
        );
    }

    #[test]
    fn app_profile_round_trips() {
        let p = AppProfile::builder()
            .name(DisplayName::from_str("Steve").unwrap())
            .default_ai_model(AiModel::Gpt5_1)
            .build();
        let json = serde_json::to_string(&p).unwrap();
        let back: AppProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
