//! Prompt-history construction (`CHAT-5`, `CHAT-9`).

use crate::model::chat::{AI_REACTOR, ChatMessage, PLAYER_REACTOR, Sender};

use crate::ai::types::{PromptMessage, Role};

/// Convert chat messages (chronological) into role-tagged prompt messages, with
/// any emoji reactions appended to the text so the character is aware of them
/// (`CHAT-5`, `CHAT-9`). Player messages map to `User`, character to `Model`.
pub fn to_history_messages(messages: &[ChatMessage]) -> Vec<PromptMessage> {
    messages
        .iter()
        .map(|m| {
            let role = match m.sender {
                Sender::Player => Role::User,
                Sender::Character { .. } => Role::Model,
            };
            PromptMessage::new(role, with_reactions(m))
        })
        .collect()
}

/// Append a compact, human-readable note of any reactions to a message's text.
fn with_reactions(message: &ChatMessage) -> String {
    let mut text = message.message.to_string();
    if message.emoji_reactions.is_empty() {
        return text;
    }
    let notes: Vec<String> = message
        .emoji_reactions
        .iter()
        .map(|(reactor, emoji)| {
            let who = match reactor.as_str() {
                PLAYER_REACTOR => "the player",
                AI_REACTOR => "you",
                other => other,
            };
            format!("{who} reacted {emoji}")
        })
        .collect();
    text.push_str(&format!("\n[{}]", notes.join("; ")));
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::chat::Reactions;
    use crate::model::ids::{ChatId, MessageId};
    use crate::model::strings::MessageString;

    fn msg(sender: Sender, text: &str, reactions: Reactions) -> ChatMessage {
        ChatMessage {
            message_id: MessageId::new(),
            version: 1,
            chat_id: ChatId::new(),
            created_at: crate::datetime::SfDateTime::now(),
            sender,
            message: MessageString::coerce(text),
            token_count: 0,
            emoji_reactions: reactions,
        }
    }

    #[test]
    fn maps_roles_and_preserves_order() {
        let history = vec![
            msg(Sender::Player, "hello", Reactions::new()),
            msg(
                Sender::Character {
                    character_id: crate::model::ids::CharacterId::new(),
                    image: None,
                },
                "hi there",
                Reactions::new(),
            ),
        ];
        let out = to_history_messages(&history);
        assert_eq!(out[0].role, Role::User);
        assert_eq!(out[0].content, "hello");
        assert_eq!(out[1].role, Role::Model);
    }

    #[test]
    fn appends_reactions_to_text() {
        let mut r = Reactions::new();
        r.set(PLAYER_REACTOR, "👍");
        let history = vec![msg(Sender::Player, "nice", r)];
        let out = to_history_messages(&history);
        assert!(out[0].content.contains("👍"));
        assert!(out[0].content.contains("the player reacted"));
    }
}
