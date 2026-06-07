//! The curated AI model registry (`AI-7`, `AI-8`).
//!
//! OpenAI-only at launch (`PROD-15`). Each entry carries a stable id (the OpenAI
//! model id string, the contract value the adapter sends), a human display name,
//! and the vendor. There is no plan gating and no pricing (cost is out of scope,
//! `STAT`). The registry defines two task defaults: a chat/narrative model and a
//! cheaper state/utility model.

use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// AI provider vendor. OpenAI is the only registered vendor at launch; the enum
/// exists so additional providers can be added behind the seam (`AI-2`).
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
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AiVendor {
    #[strum(serialize = "openai")]
    OpenAI,
}

/// A registered, selectable model. Serializes as its stable id string so the
/// persisted `ai_model` on entities is durable and provider-meaningful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiModel {
    /// Flagship chat/narrative model — the chat/narrative task default (`AI-8`).
    #[serde(rename = "gpt-5.1")]
    Gpt5_1,
    /// High-capability model option.
    #[serde(rename = "gpt-5.4")]
    Gpt5_4,
    /// Mid-tier option.
    #[serde(rename = "gpt-5.4-mini")]
    Gpt5_4Mini,
    /// Small/fast model — the state/utility task default (`AI-8`).
    #[serde(rename = "gpt-5.4-nano")]
    Gpt5_4Nano,
}

impl AiModel {
    /// The full curated registry, in display order (`AI-7`).
    pub const ALL: [AiModel; 4] = [
        AiModel::Gpt5_1,
        AiModel::Gpt5_4,
        AiModel::Gpt5_4Mini,
        AiModel::Gpt5_4Nano,
    ];

    /// The default model for chat replies and adventure narration (`AI-8`).
    pub const fn default_chat_narrative() -> AiModel {
        AiModel::Gpt5_1
    }

    /// The default model for state updates, summaries, and other background
    /// passes — a cheaper/faster model (`AI-8`).
    pub const fn default_state_utility() -> AiModel {
        AiModel::Gpt5_4Nano
    }

    /// The stable provider model id (the string sent to the provider).
    pub const fn id(&self) -> &'static str {
        match self {
            AiModel::Gpt5_1 => "gpt-5.1",
            AiModel::Gpt5_4 => "gpt-5.4",
            AiModel::Gpt5_4Mini => "gpt-5.4-mini",
            AiModel::Gpt5_4Nano => "gpt-5.4-nano",
        }
    }

    /// The human-facing display name (`AI-7`).
    pub const fn display_name(&self) -> &'static str {
        match self {
            AiModel::Gpt5_1 => "GPT-5.1",
            AiModel::Gpt5_4 => "GPT-5.4",
            AiModel::Gpt5_4Mini => "GPT-5.4 Mini",
            AiModel::Gpt5_4Nano => "GPT-5.4 Nano",
        }
    }

    /// The vendor that serves this model (`AI-7`).
    pub const fn vendor(&self) -> AiVendor {
        AiVendor::OpenAI
    }
}

impl Display for AiModel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl FromStr for AiModel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        AiModel::ALL
            .into_iter()
            .find(|m| m.id() == s)
            .ok_or_else(|| anyhow::anyhow!("unknown AI model id: {s}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_defaults_match_spec() {
        // AI-8: launch defaults are GPT-5.1 (chat) and GPT-5.4 Nano (utility).
        assert_eq!(AiModel::default_chat_narrative(), AiModel::Gpt5_1);
        assert_eq!(AiModel::default_state_utility(), AiModel::Gpt5_4Nano);
    }

    #[test]
    fn serializes_as_stable_id() {
        let json = serde_json::to_string(&AiModel::Gpt5_1).unwrap();
        assert_eq!(json, "\"gpt-5.1\"");
        let back: AiModel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, AiModel::Gpt5_1);
    }

    #[test]
    fn id_round_trips_through_from_str() {
        for m in AiModel::ALL {
            assert_eq!(AiModel::from_str(m.id()).unwrap(), m);
        }
    }

    #[test]
    fn all_models_are_openai() {
        assert!(AiModel::ALL.iter().all(|m| m.vendor() == AiVendor::OpenAI));
    }
}
