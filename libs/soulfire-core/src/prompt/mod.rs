//! Prompt assembly (`PROMPT`).
//!
//! Prompts are built from discrete, classified sections in a fixed durable-first
//! order so a stable prefix can be cached (`PROMPT-1`, `AI-4`) and the prompt
//! viewer can render exactly what is sent (`PROMPT-9`). Locked sections carry
//! Soulfire-OG's text verbatim (`PROD-7`); content toggles gate clearly-delimited
//! sub-sections by construction (`PROMPT-6`, `PROMPT-7`).
//!
//! The character-chat prompt lives here; adventure (game-master) prompts are
//! assembled by the world turn engine, which owns their JSON contracts (`WORLD`).

pub mod character;
pub mod section;
pub mod text;

pub use character::{CharacterPromptInput, build_character_prompt};
pub use section::{AssembledPrompt, PromptSection, SectionSource};
