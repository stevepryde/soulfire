//! Character-reply post-processing (`CHAT-8`). Reproduces Soulfire-OG's
//! `sanitise_message`: extract a single trailing reaction emoji (from the allowed
//! set, `DATA-6`) and normalize list-style markers (`a)`/`b)`…) to line breaks.

use std::sync::LazyLock;

use regex::Regex;

use lib_soulfire::chat::ALLOWED_EMOJIS;

/// Matches a whitespace/newline followed by a single-character list marker like
/// `a)` or `1)`, so it can be turned into a line break (Soulfire-OG `LIST_REGEX`).
static LIST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\s\n]([a-zA-Z0-9]\))").unwrap());

/// Find a trailing reaction emoji from the allowed set, if the message ends with
/// one (ignoring trailing whitespace). The allowed set has no member that is a
/// suffix of another, so a direct suffix match is unambiguous.
fn trailing_emoji(message: &str) -> Option<&'static str> {
    let trimmed = message.trim_end();
    ALLOWED_EMOJIS
        .iter()
        .find(|e| trimmed.ends_with(**e))
        .copied()
}

/// Split a character reply into its visible text and an optional trailing
/// reaction emoji (`CHAT-8`). The reaction is removed from the visible text;
/// list-style markers are normalized to `<br />` line breaks.
pub fn sanitise_reply(message: &str) -> (String, Option<&'static str>) {
    let reaction = trailing_emoji(message);

    let without_reaction = match reaction {
        Some(emoji) => message.trim_end().strip_suffix(emoji).unwrap_or(message),
        None => message,
    };

    let normalized = LIST_REGEX
        .replace_all(without_reaction, "<br />$1")
        .trim()
        .to_string();

    (normalized, reaction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_trailing_allowed_emoji() {
        // CHAT-8: a reply ending in an allowed emoji yields clean text + reaction.
        let (text, reaction) = sanitise_reply("I'm glad you're here. ❤️");
        assert_eq!(text, "I'm glad you're here.");
        assert_eq!(reaction, Some("❤️"));
    }

    #[test]
    fn leaves_text_without_emoji_untouched() {
        let (text, reaction) = sanitise_reply("Just a normal reply.");
        assert_eq!(text, "Just a normal reply.");
        assert_eq!(reaction, None);
    }

    #[test]
    fn disallowed_trailing_emoji_is_not_extracted() {
        // 🚀 is not in the allowed set, so it stays in the text.
        let (text, reaction) = sanitise_reply("To the moon 🚀");
        assert_eq!(text, "To the moon 🚀");
        assert_eq!(reaction, None);
    }

    #[test]
    fn normalizes_list_markers_to_breaks() {
        let (text, _) = sanitise_reply("Options: a) run b) hide");
        assert!(text.contains("<br />a)"));
        assert!(text.contains("<br />b)"));
    }
}
