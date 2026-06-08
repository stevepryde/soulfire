//! Chat background-pass prompts (`CHAT-3`, `CHAT-10`, `CHAT-12`), carried
//! verbatim from Soulfire-OG (`PROD-7`).

use crate::model::chat::{ChatMessage, Sender};

/// Render messages as `"Name: text"` lines for summary/state prompts.
pub fn conversation_text(
    messages: &[ChatMessage],
    player_name: &str,
    character_name: &str,
) -> String {
    messages
        .iter()
        .map(|m| {
            let name = match &m.sender {
                Sender::Player => player_name,
                Sender::Character { .. } => character_name,
            };
            format!("{name}: {}", m.message)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The rolling-summary prompt (`CHAT-10`), folding in the previous summary.
pub fn summary_prompt(existing: Option<&str>, conversation: &str) -> String {
    let mut prompt = "Summarize this conversation in 2-3 paragraphs. \
        Capture the key topics discussed, any decisions made, emotional dynamics, \
        and important context that would help continue the conversation naturally. \
        Write in third person.\n\n"
        .to_string();
    if let Some(existing) = existing {
        prompt += &format!("Previous summary:\n{existing}\n\nNew messages:\n");
    }
    prompt += conversation;
    prompt
}

/// The auto-title prompt (`CHAT-3`): a ≤5-word summary of the opening text.
pub fn title_prompt(text: &str) -> String {
    format!("summarise the following text in no more than 5 words: \"{text}\"")
}

/// The character-state update prompt (`CHAT-12`), treating the persona profile as
/// immutable and producing an evolution of the dynamic state.
pub fn state_update_prompt(
    character_name: &str,
    character_profile: &str,
    current_state: &str,
    recent_conversation: &str,
) -> String {
    format!(
        r#"You are updating the dynamic state for a character named "{character_name}" after a conversation.

## Character Profile (immutable — do not change)
{character_profile}

## Current Dynamic State
{current_state}

## Recent Conversation
{recent_conversation}

## Task
Write the UPDATED dynamic state for this character based on what just happened in the conversation.
Reflect any shifts in emotion, changes in how they feel about the player, new concerns that arose,
or old concerns that were addressed.

The state should feel like a natural evolution — not a complete rewrite. If the conversation was
light and casual, the changes might be subtle. If something significant happened, the shift should
be more pronounced.

Write in second person ("You are...", "You feel..."). Cover the same areas as before:
- Current Emotional State
- Relationship with the Player
- Current Concerns & Preoccupations
- Unresolved Threads

Keep this concise but vivid — roughly 300-500 words. Write as flowing prose, not bullet points.
Return ONLY the updated state text, nothing else."#
    )
}
