# Data Model

**Purpose:** define every persisted entity, its fields, identity, formats, relationships, and
validation. This is a data spec, so field names and formats are part of the contract: they are kept
identical to Soulfire-OG record shapes (minus account/billing/moderation fields) so prompts and
behavior port unchanged.

Storage mechanics (encryption, file location) are owned by `SEC`. AI/model enums are owned by `AI`.
Image bytes/storage are owned by `IMG`.

## Conventions

- **Typed IDs.** Every entity has a string ID with a stable type prefix (e.g. `chr_…`, `chat_…`).
  IDs are unique within their type and stable for the life of the entity.
- **Timestamps** are stored with enough precision to order events and are timezone-unambiguous (UTC).
- **Length-bounded strings.** Fields below give `(min,max)` character bounds where Soulfire-OG
  enforced them; values are trimmed and rejected (on explicit save) or clamped (on AI-generated
  updates) to fit. These bounds are contract values.
- **Schema version.** Every persisted record carries an integer `version` (starting at 1) to support
  future migrations.
- **Singletons.** Because the app is single-user (`PROD-12`), the app profile and the player profile
  are single rows; no `user_id` exists on any entity.

## Requirements

### Characters
- **DATA-1** A **Character** record has: `character_id` (prefix `chr`), `version`, `created_at`,
  `updated_at`, `last_chatted_at?`; `name` (1,100), `subtitle?` (0,500), `description` (0,1000);
  `image?` (an emoji avatar selection, see DATA-20) plus optional generated-portrait reference
  (`IMG`) and `image_transform` (pan x/y percent, zoom percent); `prompt` (0,16000) — the editable
  system prompt; `initial_message` (DATA-2); creativity controls `max_tokens` (default 2000),
  `temperature` (default 1.0), `top_p` (default 0.95), `top_k` (default 3); and AI-internal fields
  `extracted_context?` and `character_state?` (DATA-3). On save, creativity controls are clamped:
  `max_tokens` 500–5000, `temperature` 0.0–2.0, `top_p` 0.0–1.0, `top_k` 1–200.
- **DATA-2** `initial_message` is one of two variants, each wrapping a prompt/message string
  (0,16000): **Message** (a fixed opening line sent verbatim) or **Prompt** (a seed the AI uses to
  generate its opening line). The variant is part of the contract.
- **DATA-3** `extracted_context` (an AI-authored immutable persona profile) and `character_state` (an
  AI-authored evolving dynamic state) are private fields used only in prompt assembly (`PROMPT`) and
  the character-state updater (`CHAT`); they are never shown in the standard character editor (but
  may be surfaced read-only by the prompt viewer, `PROMPT`).
- **DATA-4** A Character may record its origin when extracted from a world: `source_blueprint_id?`,
  `source_adventure_id?`, `source_npc_name?` (relationships to `WorldBlueprint`/`Adventure`, DATA-10/
  DATA-12).

### Chats & messages
- **DATA-5** A **Chat** record has: `chat_id` (prefix `chat`), `version`, `started_at`, `updated_at`;
  `title` (0,200); `character_id?` (the character this chat is with, DATA-1); `participants` (the
  player and the character, each with a sender descriptor, DATA-7); `ai_model?` (the model used,
  `AI`); `chat_summary?` (rolling conversation summary); and `messages_since_summary` (counter,
  default 0). At most one chat exists per character (the per-character uniqueness from Soulfire-OG,
  now global).
- **DATA-6** A **ChatMessage** record has: `message_id` (prefix `msg`), `version`, `chat_id`
  (DATA-5), `created_at`; `sender` (DATA-7); `message` (0,4096); `token_count` (usage recorded for
  the message); and `emoji_reactions` — an insertion-ordered map of reactor → emoji. A reactor key
  is either the player or the literal `AI`. Only emojis in the allowed set are retained:
  `👍 ❤️ 😍 😂 💯 🙏 😢 ✨`.
- **DATA-7** A **sender** identifies a message author as either the **player** or a **character**
  (carrying `character_id` and the avatar to render). The single-user model collapses Soulfire-OG's
  separate "me"/"user" sender kinds into one player kind.

### Worlds: blueprints
- **DATA-8** A **WorldBlueprint** record (the reusable, authored world template) has:
  `blueprint_id` (prefix `world_blueprint`), `version`, `created_at`, `updated_at`; `title` (1,200);
  `description` (0,1000) — shown to the player, not sent to the AI; `world_prompt` (1,50000) — the
  full freeform authored world (premise, lore, locations, factions, NPCs, rules, quests, secrets,
  tone, hooks); `image?` (emoji cover selection, DATA-20) plus optional generated-cover reference
  (`IMG`) and `image_transform` (16:6 framing).
- **DATA-9** The `world_prompt` is freeform text with no enforced internal schema; the recommended
  authoring structure (Setting / Quests / Lore / NPCs / Locations / Rules) is guidance, not a
  validated format. The blueprint represents **starting conditions + hard rules + act structure +
  background**, not the live world (see WORLD).

### Worlds: adventures (playthroughs) & state
- **DATA-10** An **Adventure** record (one playthrough of one blueprint) has: `adventure_id`
  (prefix `adventure_sheet`), `version`, `created_at`, `updated_at`; `blueprint_id` (DATA-8); a
  denormalized world snapshot (`world_title?`, `world_description?`, `world_image?`,
  `world_image_transform`) for display; `world_prompt` (1,50000) — a private per-adventure copy of
  the blueprint prompt at start (so `/gm` retcons affect only this adventure, see WORLD);
  `player_name?` (0,200) and `player_attributes?` (0,5000) snapshotted from the player profile at
  start; `ai_model?` (`AI`); and the live-state and memory fields in DATA-11; plus `story_status`
  (`ongoing` | `success` | `failure`, default `ongoing`), `has_completed` (sticky once non-ongoing),
  and turn-engine bookkeeping (`ready_status`, `ready_status_updated_at?`, `diff_action_count`,
  `next_significant_event_id`, `previous_narrative?`).
- **DATA-11** Adventure live state and memory (all owned/structured by WORLD; stored as
  bounded strings here): `adventure_state` (0,50000) — compact JSON of the live world; `recent_summary`
  (0,50000) — recent events; `significant_events` (0,50000) — long-term weighted events;
  `story_summary` (0,50000) — rolling recap + recent turns.
- **DATA-12** An **AdventureMessage** record (one entry in a playthrough's turn log) has:
  `message_id` (prefix `adventure_message`), `version`, `adventure_id` (DATA-10), `created_at`;
  `message_type` (`narration` — game-master prose | `user_action` — player action |
  `game_master_request` — out-of-band player→GM | `game_master_response` — out-of-band GM→player);
  and `content` (1,10000).
- **DATA-13** A **GmProposal** record stages an out-of-band game-master change before commit (see
  WORLD): it references the adventure and the proposing response message, holds the proposed
  replacement adventure-state and/or blueprint prompt plus computed memory updates, a human-readable
  change summary (a list of `{target, path, before, after}` diff entries), and a status (`pending` |
  `accepted` | `rejected`).

### Builders (conversational create flows)
- **DATA-14** A **CharacterBuilderSession** is keyed to one character and holds an ordered message
  log (role `user`|`assistant`, content, timestamp; capped at 50, oldest dropped) and an ordered
  snapshot stack (captured `name`/`subtitle`/`description`/`prompt`/`initial_message` with timestamp;
  capped at 10, oldest dropped; duplicate-of-last skipped) backing undo (see CHAR).
- **DATA-15** A **WorldBuilderSession** is keyed to one blueprint and holds the same message log (cap
  50) and a snapshot stack of captured `title`/`description`/`world_prompt` (cap 10, dedup-last)
  backing undo (see WORLD).

### Profiles (singletons)
- **DATA-16** The **AppProfile** (one row) holds: display `name?`, `nickname?`, `primary_language`
  (default English), an optional `avatar` reference (`IMG`), and `default_ai_model?` (`AI`).
  Soulfire-OG's account/role/email fields are removed.
- **DATA-17** The **PlayerProfile** (one row) holds the default adventurer identity used when
  starting new adventures: `player_name` (0,200), `player_attributes` (0,5000), and an optional
  `prompt_extension` (0,10000) injected into adventures. Editing it affects only adventures started
  afterward.
- **DATA-18** **AppSettings** (one row) holds: the active accent **color theme** (one of seven, see
  UI), and the content/prompt toggles owned by `PROMPT` (including the adult-content toggle). The
  default color theme is Purple; the adult-content toggle defaults to off (`PROMPT-8`).
- **DATA-19** **Credentials** (provider API keys) are stored encrypted (see SEC). Each entry records
  the provider and the secret key value; the key value is never logged or shown in full after entry
  (masked display).
- **DATA-24** **InstallState** (one row) holds first-run and seed bookkeeping: `first_run_completed`
  (default false), `starter_seed_version` (default 0), and a `starter_worlds` ledger keyed by stable
  starter seed id, recording the created `blueprint_id?`, `seeded_at?`, and whether the user deleted
  that seeded starter. This state prevents first-run auto-start and starter seeding from recurring.

### Metrics
- **DATA-20a** A **UsageMetric** record (one per metered AI request) has: `metric_id` (prefix
  `metric`), `created_at`, a `label` (what the request was for, e.g. `chat_message`,
  `chat_summary`, `character_state_update`, `adventure_action`, `adventure_diff_state_update`,
  `world_builder`, `image_generation`), optional `chat_id`, optional `adventure_id`, optional
  `blueprint_id`, optional `character_id`, `input_tokens`, `output_tokens`, optional
  `cached_input_tokens`, and `ai_model` (`AI`). These associations are populated whenever the request
  belongs to that entity, so token statistics can roll up by chat, adventure, world, character,
  model, operation, and time (`STAT`). Records with zero input, cached input and output tokens are not
  written.

### Shared enums & assets
- **DATA-20** **Avatar/cover emoji selections** are fixed enumerations matching Soulfire-OG:
  characters choose from a set of ~20 emoji avatars plus the named illustrated characters bundled
  with the app (Lyra, Solas, Nova, Virel, Iris, Nikhil, Kiran, Thorne); worlds choose from ~85 emoji
  covers. The enumerations and their emoji/asset mappings are contract values reproduced from
  Soulfire-OG.
- **DATA-21** **Bundled starter content** (curated starter worlds shipped with the app, see ONB) is
  seeded as ordinary `WorldBlueprint` rows on first launch and is thereafter user-editable and
  user-deletable like any other world. The shipped starter catalog is explicit data with stable
  `seed_id`, `title`, `description`, `world_prompt`, `image`, and `image_transform` values; the launch
  catalog includes `beneath_verath` ("Beneath Verath") as the lead starter.
- **DATA-25** A fresh encrypted store is initialized with exactly one `AppProfile`, one
  `PlayerProfile`, one `AppSettings`, one `InstallState`, and no characters, chats, adventures,
  messages, metrics, or credentials until first-run setup saves the OpenAI key. Starter worlds are
  seeded only by the onboarding flow (`ONB-5`) using the `InstallState` ledger.
- **DATA-26** A **Draft** record stores unsent composer text: `draft_id` (prefix `draft`), `version`,
  `created_at`, `updated_at`, `scope` (`chat` with `chat_id` | `adventure` with `adventure_id`), and
  `content` (0,10000). At most one draft exists per scope. Drafts are local UI state, are never sent to
  the AI until the user submits them, and are deleted when their chat or adventure is deleted.

### Integrity
- **DATA-22** Deleting a Character deletes its chat and that chat's messages and builder session.
  Deleting a Chat deletes its messages but not the Character. Deleting a WorldBlueprint deletes its
  builder session; its Adventures are also removed (and their messages and any pending GmProposals).
  Deleting an Adventure deletes its messages and pending proposals only. Deleting a Chat or Adventure
  deletes its draft. No orphan rows remain.
- **DATA-23** All reads and writes are consistent within a single process: a saved change is visible
  to subsequent reads, and the persisted record is the single source of truth for what the UI shows
  and what the AI layer sends.
- **DATA-27** Every database-backed list that can grow with user content is paginated by a stable
  cursor contract rather than offset paging. The ordering for each list is total and deterministic,
  using an entity-specific sort key plus a unique tie-breaker, so paging through results visits each
  matching row once without duplicates or gaps.

## Acceptance criteria

- **AC-DATA-a** (DATA-1..3) A character round-trips through save/load with all fields intact,
  including `prompt`, `initial_message` variant, creativity controls (clamped to range), and the
  private `extracted_context`/`character_state`.
- **AC-DATA-b** (DATA-6) A chat message persists reactions; an emoji outside the allowed set is
  dropped on save; reaction order is preserved.
- **AC-DATA-c** (DATA-10, DATA-11) An adventure round-trips with its live `adventure_state` and all
  three memory layers, `story_status`, and turn bookkeeping intact; the per-adventure `world_prompt`
  copy is independent of the source blueprint.
- **AC-DATA-d** (DATA-16..19, DATA-24, DATA-25) A fresh store contains exactly one AppProfile, one
  PlayerProfile, one AppSettings, and one InstallState row, with the documented defaults and no
  characters/chats/adventures/messages/metrics/credentials until setup creates them; credentials
  persist across restarts and never appear unmasked in logs or UI.
- **AC-DATA-e** (DATA-22) After deleting a character/world/adventure, a store scan finds no messages,
  builder sessions, proposals, drafts, or chats referencing the deleted entity.
- **AC-DATA-f** (DATA-20a) Each metered AI call writes exactly one UsageMetric with correct label,
  token counts, and applicable entity associations; a zero/zero call writes none.
- **AC-DATA-g** (DATA-26) Saving a chat or adventure draft replaces any prior draft for that scope;
  reopening the same chat/adventure restores the draft; submitting or deleting the parent clears it.
- **AC-DATA-h** (DATA-27) Paging through characters, blueprints, adventures, and other growing list
  surfaces with a small page size returns the same ordered entity IDs as one large fetch, without
  duplicates or gaps, including when multiple rows share the primary sort value.

## Design notes (non-normative)

- The persisted shapes mirror Soulfire-OG's `*Record` structs in `lib-soulfire`/`soulfire-api`
  (`CharacterRecord`, `ChatRecord`, `ChatMessageRecord`, `WorldBlueprintRecord`,
  `AdventureSheetRecord`, `AdventureMessageRecord`, `MetricsRecord`, builder sessions, GM proposal
  record). Keeping the field names eases porting prompt-assembly code verbatim.
- Bounded-string newtypes (the Soulfire-OG `string_type!` pattern) are a convenient way to enforce
  DATA length bounds in one place.
- The store is relational (one table per entity is the natural mapping); the JSON-string state/memory
  fields on Adventure are opaque blobs to the store and parsed only by the WORLD layer.
