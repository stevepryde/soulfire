//! Model-selection precedence (`AI-9`) and token estimation (`AI-16`).

use lib_soulfire::ai_model::AiModel;

/// Resolve the model for an operation by precedence (`AI-9`): the entity's stored
/// model if set, else the app profile's default, else the registry task default.
pub fn resolve_model(
    entity_model: Option<AiModel>,
    profile_default: Option<AiModel>,
    task_default: AiModel,
) -> AiModel {
    entity_model.or(profile_default).unwrap_or(task_default)
}

/// A rough token-count estimate for displaying prompt sizes in the prompt viewer
/// (`AI-16`, `PROMPT-11`). This is an approximation of OpenAI's tokenizer using
/// the well-known ~4-characters-per-token heuristic; it is used for display only,
/// never for billing, and the provider's reported usage is authoritative for
/// metering (`STAT`).
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Blend a chars/4 estimate with a word-count floor so very short or
    // whitespace-heavy text still estimates sensibly.
    let chars = text.chars().count();
    let by_chars = chars.div_ceil(4);
    let by_words = text.split_whitespace().count();
    by_chars.max(by_words).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_prefers_entity_then_profile_then_default() {
        // AI-9 order.
        assert_eq!(
            resolve_model(Some(AiModel::Gpt5_4), Some(AiModel::Gpt5_4Mini), AiModel::Gpt5_1),
            AiModel::Gpt5_4
        );
        assert_eq!(
            resolve_model(None, Some(AiModel::Gpt5_4Mini), AiModel::Gpt5_1),
            AiModel::Gpt5_4Mini
        );
        assert_eq!(
            resolve_model(None, None, AiModel::Gpt5_1),
            AiModel::Gpt5_1
        );
    }

    #[test]
    fn token_estimate_grows_with_length() {
        assert_eq!(estimate_tokens(""), 0);
        let short = estimate_tokens("hello world");
        let long = estimate_tokens(&"hello world ".repeat(50));
        assert!(long > short);
        assert!(short >= 2);
    }
}
