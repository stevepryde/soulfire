//! Adventure composer input parsing (`WORLD-15`, `UI-12`, `TEST-6`).

/// The classified kind of a composer submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnInput {
    /// An ordinary in-world action.
    Action(String),
    /// An out-of-band game-master request with text.
    GmRequest(String),
    /// `/gm` with no request text — warn the user to add a request.
    GmEmpty,
    /// An unknown `/x` slash command — warn "unknown command".
    Unknown(String),
}

/// Parse composer input (`WORLD-15`): `/gm <text>` is a GM request; `/gm` alone
/// warns; any other leading `/command` is unknown; plain text is an action.
pub fn parse_turn_input(raw: &str) -> TurnInput {
    let trimmed = raw.trim();
    if let Some(rest) = trimmed.strip_prefix("/gm") {
        // Must be exactly `/gm` or `/gm <text>` (not e.g. `/gmx`).
        if rest.is_empty() {
            return TurnInput::GmEmpty;
        }
        if let Some(first) = rest.chars().next() {
            if first.is_whitespace() {
                let request = rest.trim();
                return if request.is_empty() {
                    TurnInput::GmEmpty
                } else {
                    TurnInput::GmRequest(request.to_string())
                };
            }
        }
    }
    if trimmed.starts_with('/') {
        let command = trimmed
            .split_whitespace()
            .next()
            .unwrap_or(trimmed)
            .to_string();
        return TurnInput::Unknown(command);
    }
    TurnInput::Action(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gm_request_with_text() {
        assert_eq!(
            parse_turn_input("/gm skip to morning"),
            TurnInput::GmRequest("skip to morning".to_string())
        );
    }

    #[test]
    fn gm_with_no_text_warns() {
        assert_eq!(parse_turn_input("/gm"), TurnInput::GmEmpty);
        assert_eq!(parse_turn_input("/gm   "), TurnInput::GmEmpty);
    }

    #[test]
    fn unknown_slash_command_warns() {
        assert_eq!(
            parse_turn_input("/help me"),
            TurnInput::Unknown("/help".to_string())
        );
        // `/gmx` is not the gm command.
        assert_eq!(
            parse_turn_input("/gmx"),
            TurnInput::Unknown("/gmx".to_string())
        );
    }

    #[test]
    fn plain_text_is_an_action() {
        assert_eq!(
            parse_turn_input("  look around  "),
            TurnInput::Action("look around".to_string())
        );
    }
}
