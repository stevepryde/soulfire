//! AI image generation for character portraits and world covers (`IMG`).
//!
//! Generation derives a prompt from the entity, calls the provider's image
//! generation (`AI`), and stores the bytes encrypted in the store (`IMG-4`,
//! `SEC-3`). Regeneration bumps a cache-bust version so the UI re-renders
//! (`IMG-3`); clearing returns to the emoji avatar/cover (`IMG-8`). A failed
//! generation leaves any prior image in place (`IMG-2`). The crop/transform is
//! persisted on the entity and edited in the UI (`IMG-7`).

use std::sync::Arc;

use lib_soulfire::ai_model::AiModel;
use lib_soulfire::ids::{CharacterId, WorldBlueprintId};
use lib_soulfire::images::StoredImageRef;
use lib_soulfire::metric::{MetricLabel, UsageMetric};

use crate::ai::registry::resolve_model;
use crate::ai::service::AiService;
use crate::ai::types::{ImageRequest, Usage};
use crate::clock::Clock;
use crate::error::{CoreError, CoreResult};
use crate::store::{ImageOwnerKind, Store};

/// Drives portrait/cover generation and storage.
#[derive(Clone)]
pub struct ImageEngine {
    store: Arc<Store>,
    ai: AiService,
    clock: Arc<dyn Clock>,
}

impl ImageEngine {
    pub fn new(store: Arc<Store>, ai: AiService, clock: Arc<dyn Clock>) -> Self {
        ImageEngine { store, ai, clock }
    }

    /// Generate (or regenerate) a character portrait (`IMG-1`, `IMG-3`). On
    /// success the new image replaces any prior one and the version bumps; on
    /// failure the prior image is left in place (`IMG-2`).
    pub async fn generate_character_portrait(
        &self,
        character_id: &CharacterId,
    ) -> CoreResult<StoredImageRef> {
        let mut character = self
            .store
            .character(character_id)?
            .ok_or_else(|| CoreError::NotFound(character_id.to_string()))?;
        let prompt = character_portrait_prompt(
            character.name.as_str(),
            character.description.as_str(),
            character.prompt.as_str(),
        );
        let image = self.generate(prompt).await?;
        let owner = character_id.to_string();
        let next = match character.portrait {
            Some(prev) => prev.bumped(),
            None => StoredImageRef::new(),
        };
        self.store
            .put_image(ImageOwnerKind::Character, &owner, &image.mime, next.version, &image.bytes)?;
        character.portrait = Some(next);
        character.updated_at = self.clock.now();
        self.store.save_character(&character)?;
        self.meter(image.usage, Some(character_id), None)?;
        Ok(next)
    }

    /// Generate (or regenerate) a world cover (`IMG-1`, `IMG-3`).
    pub async fn generate_world_cover(
        &self,
        blueprint_id: &WorldBlueprintId,
    ) -> CoreResult<StoredImageRef> {
        let mut blueprint = self
            .store
            .blueprint(blueprint_id)?
            .ok_or_else(|| CoreError::NotFound(blueprint_id.to_string()))?;
        let prompt = world_cover_prompt(
            blueprint.title.as_str(),
            blueprint.description.as_str(),
            blueprint.world_prompt.as_str(),
        );
        let image = self.generate(prompt).await?;
        let owner = blueprint_id.to_string();
        let next = match blueprint.cover {
            Some(prev) => prev.bumped(),
            None => StoredImageRef::new(),
        };
        self.store
            .put_image(ImageOwnerKind::World, &owner, &image.mime, next.version, &image.bytes)?;
        blueprint.cover = Some(next);
        blueprint.updated_at = self.clock.now();
        self.store.save_blueprint(&blueprint)?;
        self.meter(image.usage, None, Some(blueprint_id))?;
        Ok(next)
    }

    /// Store a user-uploaded image as a character portrait (`IMG-6`). Stored and
    /// protected identically to generated images.
    pub fn set_character_portrait_bytes(
        &self,
        character_id: &CharacterId,
        mime: &str,
        bytes: &[u8],
    ) -> CoreResult<StoredImageRef> {
        let mut character = self
            .store
            .character(character_id)?
            .ok_or_else(|| CoreError::NotFound(character_id.to_string()))?;
        let next = match character.portrait {
            Some(prev) => prev.bumped(),
            None => StoredImageRef::new(),
        };
        self.store
            .put_image(ImageOwnerKind::Character, &character_id.to_string(), mime, next.version, bytes)?;
        character.portrait = Some(next);
        character.updated_at = self.clock.now();
        self.store.save_character(&character)?;
        Ok(next)
    }

    /// Clear a character's portrait back to its emoji avatar (`IMG-3`, `IMG-8`).
    pub fn clear_character_portrait(&self, character_id: &CharacterId) -> CoreResult<()> {
        let mut character = self
            .store
            .character(character_id)?
            .ok_or_else(|| CoreError::NotFound(character_id.to_string()))?;
        self.store
            .delete_image(ImageOwnerKind::Character, &character_id.to_string())?;
        character.portrait = None;
        character.updated_at = self.clock.now();
        self.store.save_character(&character)?;
        Ok(())
    }

    /// Clear a world cover back to its emoji (`IMG-3`, `IMG-8`).
    pub fn clear_world_cover(&self, blueprint_id: &WorldBlueprintId) -> CoreResult<()> {
        let mut blueprint = self
            .store
            .blueprint(blueprint_id)?
            .ok_or_else(|| CoreError::NotFound(blueprint_id.to_string()))?;
        self.store
            .delete_image(ImageOwnerKind::World, &blueprint_id.to_string())?;
        blueprint.cover = None;
        blueprint.updated_at = self.clock.now();
        self.store.save_blueprint(&blueprint)?;
        Ok(())
    }

    async fn generate(&self, prompt: String) -> CoreResult<crate::ai::types::ImageResponse> {
        let model = resolve_model(None, self.store.app_profile()?.default_ai_model, AiModel::default_chat_narrative());
        Ok(self.ai.generate_image(ImageRequest { model, prompt }).await?)
    }

    fn meter(
        &self,
        usage: Usage,
        character_id: Option<&CharacterId>,
        blueprint_id: Option<&WorldBlueprintId>,
    ) -> CoreResult<()> {
        let metric = UsageMetric::builder()
            .created_at(self.clock.now())
            .label(MetricLabel::ImageGeneration)
            .maybe_character_id(character_id.cloned())
            .maybe_blueprint_id(blueprint_id.cloned())
            .input_tokens(usage.input_tokens)
            .output_tokens(usage.output_tokens)
            .maybe_cached_input_tokens(usage.cached_input_tokens)
            .ai_model(AiModel::default_chat_narrative())
            .build();
        self.store.save_metric(&metric)
    }
}

/// Build a portrait generation prompt from a character (`IMG-1`). Kept small and
/// separate so a future provider can supply images without touching editors.
pub fn character_portrait_prompt(name: &str, description: &str, prompt: &str) -> String {
    let detail = if !description.is_empty() {
        description
    } else {
        truncate(prompt, 500)
    };
    format!(
        "A striking character portrait of {name}. {detail} Head-and-shoulders framing, expressive, painterly digital art, dramatic lighting."
    )
}

/// Build a world-cover generation prompt from a blueprint (`IMG-1`).
pub fn world_cover_prompt(title: &str, description: &str, world_prompt: &str) -> String {
    let detail = if !description.is_empty() {
        description
    } else {
        truncate(world_prompt, 500)
    };
    format!(
        "Wide cinematic cover art for the world \"{title}\". {detail} Atmospheric establishing shot, rich detail, no text."
    )
}

fn truncate(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_prompt_prefers_description_falls_back_to_prompt() {
        let with_desc = character_portrait_prompt("Lyra", "a calm forest guide", "long prompt");
        assert!(with_desc.contains("Lyra"));
        assert!(with_desc.contains("calm forest guide"));
        let no_desc = character_portrait_prompt("Nox", "", "a shadowy figure who haunts the docks");
        assert!(no_desc.contains("shadowy figure"));
    }

    #[test]
    fn cover_prompt_includes_title() {
        let p = world_cover_prompt("Beneath Verath", "a drowned city", "");
        assert!(p.contains("Beneath Verath"));
        assert!(p.contains("drowned city"));
    }
}
