//! Shared Soulfire domain models and types.
//!
//! The persisted entities, typed IDs, bounded strings, and enums that make up
//! Soulfire's data contract (`specs/01-data-model.md`). Ported from Soulfire-OG's
//! shared model crate with the single-user collapse applied: no `user_id`, no
//! visibility, no account/billing/moderation; profiles and settings are
//! singleton rows. Field names and formats are kept identical to Soulfire-OG so
//! prompts and behavior port unchanged (`PROD-7`).

pub mod ai_model;
pub mod character;
pub mod chat;
pub mod credentials;
pub mod draft;
pub mod ids;
pub mod images;
pub mod install;
pub mod metric;
pub mod profile;
pub mod settings;
pub mod strings;
pub mod world;

pub use ai_model::{AiModel, AiVendor};
pub use character::{
    Character, CharacterBuilderMessage, CharacterBuilderRole, CharacterBuilderSession,
    CharacterBuilderSnapshot, CreativityControls, InitialMessage,
};
pub use chat::{ALLOWED_EMOJIS, Chat, ChatMessage, Reactions, Sender};
pub use credentials::ProviderCredential;
pub use draft::{Draft, DraftScope};
pub use images::{CharacterImage, ImageTransform, StoredImageRef, WorldImage};
pub use install::{InstallState, StarterSeedRecord};
pub use metric::{MetricLabel, UsageMetric};
pub use profile::{AppProfile, Language, PlayerProfile};
pub use settings::{AppSettings, ColorTheme, ContentToggle, ContentToggles};
pub use world::{
    Adventure, AdventureMessage, AdventureMessageType, AdventureReadyStatus, GmChangeTarget,
    GmDiffEntry, GmProposal, GmProposalStatus, StoryStatus, WorldBlueprint, WorldBuilderMessage,
    WorldBuilderRole, WorldBuilderSession, WorldBuilderSnapshot,
};
