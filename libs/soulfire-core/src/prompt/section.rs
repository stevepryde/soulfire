//! The sectioned-prompt model (`PROMPT-1`, `PROMPT-2`, `PROMPT-9`).
//!
//! A prompt is assembled from discrete named sections in a fixed, durable-first
//! order. Each section is classified locked vs editable and labeled with its
//! source, so the prompt viewer can render exactly the structure the builder
//! emits — guaranteeing the view matches what is sent.

/// Where a section's body comes from (`PROMPT-9` source labeling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionSource {
    /// The originating world's prompt (world-linked character).
    WorldPrompt,
    /// The character's AI-authored persona profile (`extracted_context`).
    ExtractedContext,
    /// The user-authored `Character.prompt` (the one editable section).
    AuthoredCharacterPrompt,
    /// The locked behavior-instructions block.
    BehaviorInstructions,
    /// The locked reactions rule.
    Reactions,
    /// The live world-state block (adventure state + story so far).
    WorldState,
    /// The character's AI-authored evolving dynamic state (`character_state`).
    DynamicState,
    /// Locked game-master instructions (adventure prompts).
    GameMasterInstructions,
    /// A live adventure memory/state block (adventure prompts).
    AdventureContext,
    /// The player profile / authored world prompt within an adventure.
    AuthoredWorldPrompt,
}

/// One named, classified prompt section (`PROMPT-2`, `PROMPT-9`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSection {
    /// The section's contract anchor header (e.g. `## How to Be This Character`).
    pub header: String,
    /// The section body (without the header).
    pub body: String,
    /// Locked (required, not user-editable) vs editable (`PROMPT-2`).
    pub locked: bool,
    /// The backing source, for viewer labeling (`PROMPT-9`).
    pub source: SectionSource,
}

impl PromptSection {
    pub fn locked(
        header: impl Into<String>,
        body: impl Into<String>,
        source: SectionSource,
    ) -> Self {
        PromptSection {
            header: header.into(),
            body: body.into(),
            locked: true,
            source,
        }
    }

    pub fn editable(
        header: impl Into<String>,
        body: impl Into<String>,
        source: SectionSource,
    ) -> Self {
        PromptSection {
            header: header.into(),
            body: body.into(),
            locked: false,
            source,
        }
    }

    /// The section rendered as it appears in the assembled prompt: header, blank
    /// line, body.
    pub fn rendered(&self) -> String {
        if self.body.is_empty() {
            self.header.clone()
        } else {
            format!("{}\n\n{}", self.header, self.body)
        }
    }
}

/// An assembled, ordered list of sections forming a cacheable instructions prefix
/// (`PROMPT-1`, `AI-4`). The volatile message history and current message are
/// carried separately by the engines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssembledPrompt {
    pub sections: Vec<PromptSection>,
}

impl AssembledPrompt {
    pub fn new(sections: Vec<PromptSection>) -> Self {
        AssembledPrompt { sections }
    }

    /// The full instructions string: every section rendered in order, joined by
    /// blank lines (the stable, cache-eligible prefix, `AI-4`).
    pub fn instructions(&self) -> String {
        self.sections
            .iter()
            .map(PromptSection::rendered)
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// The ordered (header, locked) pairs, for asserting structure in tests and
    /// rendering the viewer (`PROMPT-9`).
    pub fn outline(&self) -> Vec<(String, bool)> {
        self.sections
            .iter()
            .map(|s| (s.header.clone(), s.locked))
            .collect()
    }
}
