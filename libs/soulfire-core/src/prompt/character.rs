//! Character-chat prompt assembly (`PROMPT-3`, `PROMPT-4`, `PROMPT-6`).

use crate::model::chat::ALLOWED_EMOJIS;
use crate::model::settings::ContentToggles;

use super::section::{AssembledPrompt, PromptSection, SectionSource};
use super::text;

/// Inputs for assembling a character-chat prompt. The engine gathers these from
/// the character and any linked world/adventure (`CHAT`).
#[derive(Debug, Clone, Default)]
pub struct CharacterPromptInput<'a> {
    /// The user-authored `Character.prompt` (editable, may be empty).
    pub character_prompt: &'a str,
    /// The AI-authored persona profile (`extracted_context`), if any.
    pub extracted_context: Option<&'a str>,
    /// The AI-authored evolving dynamic state (`character_state`), if any.
    pub character_state: Option<&'a str>,
    /// Whether the character originated from a world (adventure-linked).
    pub is_adventure_linked: bool,
    /// The originating world's prompt, if world-linked.
    pub world_context: Option<&'a str>,
    /// The linked adventure's live state, if world-linked.
    pub world_state: Option<&'a str>,
    /// The linked adventure's story summary, if non-empty.
    pub story_so_far: Option<&'a str>,
    /// Content toggles (the adult-content stance, `PROMPT-6`).
    pub toggles: ContentToggles,
}

/// Build the reactions-rule body over the allowed emoji set (`PROMPT-3` e,
/// `DATA-6`).
fn reactions_body() -> String {
    format!(
        "You may optionally end your message with a single emoji from this set: {}. \
         Place it as the very last character of your response. Only react when it feels \
         natural — most messages don't need one.",
        ALLOWED_EMOJIS.join(" ")
    )
}

/// Assemble the behavior-instructions body, gating the mature-roleplay stance on
/// the adult-content toggle (`PROMPT-4`, `PROMPT-6`, `PROMPT-7`).
fn behavior_body(is_adventure_linked: bool, adult_content: bool) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if is_adventure_linked {
        parts.push(text::BEHAVIOR_ADVENTURE_LINKED_INTRO);
    }
    parts.push(text::BEHAVIOR_VOICE_AND_PRESENCE);
    parts.push(text::BEHAVIOR_DEPTH_AND_ENGAGEMENT);
    if adult_content {
        parts.push(text::BEHAVIOR_MATURE_ROLEPLAY);
    }
    parts.push(text::BEHAVIOR_WHAT_NOT_TO_DO);
    parts.push(text::BEHAVIOR_RESPONSE_LENGTH);
    parts.join("\n\n")
}

/// Assemble the character-chat instructions prompt in the fixed `PROMPT-3` order.
/// Optional sections are present only when their backing data exists; the order
/// of present sections is unchanged.
pub fn build_character_prompt(input: &CharacterPromptInput) -> AssembledPrompt {
    let mut sections = Vec::new();

    // (a) World Context — locked, optional.
    if let Some(world) = input.world_context.filter(|s| !s.is_empty()) {
        sections.push(PromptSection::locked(
            text::H_WORLD_CONTEXT,
            format!("{}\n\n{}", text::WORLD_CONTEXT_INTRO, world),
            SectionSource::WorldPrompt,
        ));
    }

    // (b) Your Character Profile — locked, optional.
    if let Some(extracted) = input.extracted_context.filter(|s| !s.is_empty()) {
        sections.push(PromptSection::locked(
            text::H_CHARACTER_PROFILE,
            format!("{}\n\n{}", text::CHARACTER_PROFILE_INTRO, extracted),
            SectionSource::ExtractedContext,
        ));
    }

    // (c) Character Prompt — editable (the primary authored persona).
    if !input.character_prompt.is_empty() {
        sections.push(PromptSection::editable(
            text::H_CHARACTER_PROMPT,
            input.character_prompt,
            SectionSource::AuthoredCharacterPrompt,
        ));
    }

    // (d) Behavior instructions — locked (mature stance gated by toggle).
    sections.push(PromptSection::locked(
        text::H_BEHAVIOR,
        behavior_body(input.is_adventure_linked, input.toggles.adult_content),
        SectionSource::BehaviorInstructions,
    ));

    // (e) Reactions — locked.
    sections.push(PromptSection::locked(
        text::H_REACTIONS,
        reactions_body(),
        SectionSource::Reactions,
    ));

    // (f) World-state block — locked, optional (world-linked).
    if let Some(state) = input.world_state.filter(|s| !s.is_empty()) {
        sections.push(PromptSection::locked(
            text::H_WORLD_STATE,
            format!("{}\n\n{}", text::WORLD_STATE_INTRO, state),
            SectionSource::WorldState,
        ));
        if let Some(story) = input.story_so_far.filter(|s| !s.is_empty()) {
            sections.push(PromptSection::locked(
                text::H_STORY_SO_FAR,
                format!("{}\n\n{}", text::STORY_SO_FAR_INTRO, story),
                SectionSource::WorldState,
            ));
        }
    }

    // (g) Your Current State — locked, optional.
    if let Some(dynamic) = input.character_state.filter(|s| !s.is_empty()) {
        sections.push(PromptSection::locked(
            text::H_CURRENT_STATE,
            format!("{}\n\n{}", text::CURRENT_STATE_INTRO, dynamic),
            SectionSource::DynamicState,
        ));
    }

    AssembledPrompt::new(sections)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> CharacterPromptInput<'static> {
        CharacterPromptInput {
            character_prompt: "You are Lyra, a wandering guide.",
            ..Default::default()
        }
    }

    #[test]
    fn plain_character_has_core_sections_in_order() {
        // AC-PROMPT-a: optional sections absent, present order unchanged.
        let p = build_character_prompt(&plain());
        let outline = p.outline();
        assert_eq!(
            outline,
            vec![
                ("## Character Prompt".to_string(), false), // editable
                ("## How to Be This Character".to_string(), true),
                ("## Reactions".to_string(), true),
            ]
        );
    }

    #[test]
    fn world_linked_character_includes_optional_sections_in_order() {
        // AC-PROMPT-a: world-linked character with dynamic state.
        let input = CharacterPromptInput {
            character_prompt: "persona",
            extracted_context: Some("a quiet scholar"),
            character_state: Some("anxious, hopeful"),
            is_adventure_linked: true,
            world_context: Some("the drowned city"),
            world_state: Some("{\"day\":3}"),
            story_so_far: Some("They escaped the flood."),
            toggles: ContentToggles::default(),
        };
        let headers: Vec<String> = build_character_prompt(&input)
            .outline()
            .into_iter()
            .map(|(h, _)| h)
            .collect();
        assert_eq!(
            headers,
            vec![
                "## World Context",
                "## Your Character Profile",
                "## Character Prompt",
                "## How to Be This Character",
                "## Reactions",
                "## Current State of the World",
                "## Story So Far",
                "## Your Current State",
            ]
        );
    }

    #[test]
    fn adult_toggle_off_omits_mature_stance_on_present_otherwise() {
        // AC-PROMPT-b: adult off -> mature stance absent; on -> present.
        let off = build_character_prompt(&plain()).instructions();
        assert!(!off.contains("### Mature Roleplay"));
        assert!(!off.contains("explicit erotic language"));

        let input = CharacterPromptInput {
            toggles: ContentToggles {
                adult_content: true,
            },
            ..plain()
        };
        let on = build_character_prompt(&input).instructions();
        assert!(on.contains("### Mature Roleplay"));
        assert!(on.contains("explicit erotic language"));
    }

    #[test]
    fn structural_sections_present_with_toggles_off() {
        // AC-PROMPT-c: wrappers + reactions rule remain even with toggles off.
        let p = build_character_prompt(&plain());
        let text = p.instructions();
        assert!(text.contains("## How to Be This Character"));
        assert!(text.contains("## Reactions"));
        assert!(text.contains("single emoji from this set:"));
        assert!(text.contains("👍"));
    }

    #[test]
    fn only_character_prompt_is_editable() {
        // PROMPT-2/AC-PROMPT-d: exactly one editable section (the authored prompt).
        let input = CharacterPromptInput {
            extracted_context: Some("x"),
            character_state: Some("y"),
            world_context: Some("z"),
            world_state: Some("w"),
            ..plain()
        };
        let p = build_character_prompt(&input);
        let editable: Vec<_> = p.sections.iter().filter(|s| !s.locked).collect();
        assert_eq!(editable.len(), 1);
        assert_eq!(editable[0].source, SectionSource::AuthoredCharacterPrompt);
    }

    #[test]
    fn adventure_linked_intro_only_when_linked() {
        let not_linked = build_character_prompt(&plain()).instructions();
        assert!(!not_linked.contains("You exist outside the story now"));
        let input = CharacterPromptInput {
            is_adventure_linked: true,
            ..plain()
        };
        let linked = build_character_prompt(&input).instructions();
        assert!(linked.contains("You exist outside the story now"));
    }
}
