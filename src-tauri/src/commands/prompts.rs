use std::str::FromStr;

use serde::{Deserialize, Serialize};
use soulfire_core::ai::registry::estimate_tokens;
use soulfire_core::error::CoreResult;
use soulfire_core::model::character::Character;
use soulfire_core::model::ids::CharacterId;
use soulfire_core::model::strings::CharacterPrompt;
use soulfire_core::prompt::{
    CharacterPromptInput, PromptSection, SectionSource, build_character_prompt,
};
use soulfire_core::store::Store;
use tauri::State;

use crate::error::CommandError;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSectionSourceDto {
    WorldPrompt,
    ExtractedContext,
    AuthoredCharacterPrompt,
    BehaviorInstructions,
    Reactions,
    WorldState,
    DynamicState,
    GameMasterInstructions,
    AdventureContext,
    AuthoredWorldPrompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptSectionDto {
    pub header: String,
    pub body: String,
    pub locked: bool,
    pub source: PromptSectionSourceDto,
    pub token_estimate: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptViewDto {
    pub sections: Vec<PromptSectionDto>,
    pub full_prompt: String,
    pub token_estimate: usize,
}

struct OwnedCharacterPromptInput {
    character_prompt: String,
    extracted_context: Option<String>,
    character_state: Option<String>,
    is_adventure_linked: bool,
    world_context: Option<String>,
    world_state: Option<String>,
    story_so_far: Option<String>,
    toggles: soulfire_core::model::settings::ContentToggles,
}

impl OwnedCharacterPromptInput {
    fn as_ref(&self) -> CharacterPromptInput<'_> {
        CharacterPromptInput {
            character_prompt: &self.character_prompt,
            extracted_context: self.extracted_context.as_deref(),
            character_state: self.character_state.as_deref(),
            is_adventure_linked: self.is_adventure_linked,
            world_context: self.world_context.as_deref(),
            world_state: self.world_state.as_deref(),
            story_so_far: self.story_so_far.as_deref(),
            toggles: self.toggles,
        }
    }
}

fn section_source_dto(source: SectionSource) -> PromptSectionSourceDto {
    match source {
        SectionSource::WorldPrompt => PromptSectionSourceDto::WorldPrompt,
        SectionSource::ExtractedContext => PromptSectionSourceDto::ExtractedContext,
        SectionSource::AuthoredCharacterPrompt => PromptSectionSourceDto::AuthoredCharacterPrompt,
        SectionSource::BehaviorInstructions => PromptSectionSourceDto::BehaviorInstructions,
        SectionSource::Reactions => PromptSectionSourceDto::Reactions,
        SectionSource::WorldState => PromptSectionSourceDto::WorldState,
        SectionSource::DynamicState => PromptSectionSourceDto::DynamicState,
        SectionSource::GameMasterInstructions => PromptSectionSourceDto::GameMasterInstructions,
        SectionSource::AdventureContext => PromptSectionSourceDto::AdventureContext,
        SectionSource::AuthoredWorldPrompt => PromptSectionSourceDto::AuthoredWorldPrompt,
    }
}

fn section_dto(section: PromptSection) -> PromptSectionDto {
    let token_estimate = estimate_tokens(&section.rendered());
    PromptSectionDto {
        header: section.header,
        body: section.body,
        locked: section.locked,
        source: section_source_dto(section.source),
        token_estimate,
    }
}

fn prompt_view(input: OwnedCharacterPromptInput) -> PromptViewDto {
    let assembled = build_character_prompt(&input.as_ref());
    let full_prompt = assembled.instructions();
    PromptViewDto {
        sections: assembled.sections.into_iter().map(section_dto).collect(),
        token_estimate: estimate_tokens(&full_prompt),
        full_prompt,
    }
}

fn character_prompt_input(
    store: &Store,
    character: &Character,
) -> CoreResult<OwnedCharacterPromptInput> {
    let mut input = OwnedCharacterPromptInput {
        character_prompt: character.prompt.to_string(),
        extracted_context: character.extracted_context.as_ref().map(|c| c.to_string()),
        character_state: character.character_state.as_ref().map(|c| c.to_string()),
        is_adventure_linked: character.source_adventure_id.is_some(),
        world_context: None,
        world_state: None,
        story_so_far: None,
        toggles: store.app_settings()?.content_toggles,
    };
    if let Some(bp_id) = &character.source_blueprint_id {
        if let Some(bp) = store.blueprint(bp_id)? {
            input.world_context = Some(bp.world_prompt.to_string());
        }
    }
    if let Some(adv_id) = &character.source_adventure_id {
        if let Some(adv) = store.adventure(adv_id)? {
            input.world_state = Some(adv.adventure_state.to_string());
            if !adv.story_summary.as_str().is_empty() {
                input.story_so_far = Some(adv.story_summary.to_string());
            }
        }
    }
    Ok(input)
}

#[tauri::command]
pub async fn get_character_prompt_view(
    character_id: CharacterId,
    state: State<'_, AppState>,
) -> Result<PromptViewDto, CommandError> {
    state
        .with_store(move |store| {
            let character = store
                .character(&character_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(character_id.to_string()))?;
            Ok(prompt_view(character_prompt_input(store, &character)?))
        })
        .await
}

#[tauri::command]
pub async fn save_character_prompt_section(
    character_id: CharacterId,
    source: PromptSectionSourceDto,
    body: String,
    state: State<'_, AppState>,
) -> Result<PromptViewDto, CommandError> {
    if source != PromptSectionSourceDto::AuthoredCharacterPrompt {
        return Err(CommandError::InvalidInput(
            "only the authored character prompt section is editable".to_string(),
        ));
    }
    let prompt = CharacterPrompt::from_str(&body)
        .map_err(|err| CommandError::InvalidInput(err.to_string()))?;
    state
        .with_store(move |store| {
            let mut character = store
                .character(&character_id)?
                .ok_or_else(|| soulfire_core::CoreError::NotFound(character_id.to_string()))?;
            character.prompt = prompt;
            store.save_character(&character)?;
            Ok(prompt_view(character_prompt_input(store, &character)?))
        })
        .await
}
