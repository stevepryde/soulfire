# Soulfire-OG Feature Inventory

Status: Phase 2 roadmap artifact. This document is descriptive, not normative. The specs remain the
source of truth for product behavior.

Last updated: 2026-06-08.

## Purpose

This is the concrete parity checklist for the Tauri v2 + React rebuild. It maps Soulfire-OG
(`~/projects/app-world/soulfire`) to this repo and names every major item as one of:

- **Ported**: implemented in the local Rust core with no intentional behavior change beyond the
  spec's local-only removals.
- **Adapted**: implemented with a named local/native change.
- **Removed**: intentionally out of scope for the single-user local app.
- **Missing**: required by the specs but not implemented here yet.
- **Untrusted**: code exists, but the roadmap says it needs fixture, golden, bridge, or UI validation
  before it should be treated as complete.

## Source Scan

OG source areas scanned:

- `libs/lib-soulfire/src/models`
- `libs/lib-soulfire/src/api`
- `bins/soulfire-api/src/db`
- `bins/soulfire-api/src/routes`
- `bins/soulfire-api/src/services`
- `bins/soulfire-ui/src/pages`
- `bins/soulfire-ui/src/components`
- `bins/soulfire-ui/src/hooks`
- `bins/soulfire-ui/input.css`
- `manifest/`
- `res/`

Local source areas scanned:

- `libs/soulfire-core/src/model`
- `libs/soulfire-core/src/store`
- `libs/soulfire-core/src/ai`
- `libs/soulfire-core/src/chat`
- `libs/soulfire-core/src/character`
- `libs/soulfire-core/src/world`
- `libs/soulfire-core/src/image`
- `libs/soulfire-core/tests`
- `libs/ai-client/src`

## Current Phase Summary

| Roadmap area | Status | Notes |
| --- | --- | --- |
| Phase 0: quarantine current state | Done | Dioxus UI was removed from this repo; the workspace currently contains Rust core crates only. |
| Phase 1: spec update pass | Done enough for implementation | Specs describe the Tauri v2 + React stack, local-only removals, cursor pagination, and React/Tauri testing expectations. |
| Phase 2: feature inventory and diff | Done here | This document is the first concrete OG parity matrix. Keep it current as work lands. |
| Phase 3: Rust core | Partial | A broad Rust core exists, including encrypted SQLite, models, chat, worlds, builders, images, metrics, pagination, request-level config coverage, representative prompt hash snapshots, and OG-to-local data fixtures. Some trust checks remain. |
| Phase 4: Tauri bridge | Partial | `src-tauri` now provides the Tauri v2 crate/config/capability scaffold, narrow app command permissions, async typed setup/unlock/profile/settings/credential commands, the first character chat command/event slice, and the first event DTO vocabulary. World/adventure feature commands remain. |
| Phase 5: React UI fidelity port | Missing | No React/Vite/Tailwind app exists in this checkout. OG UI remains reference-only. |
| Phase 6: desktop/mobile readiness | Missing | No Tauri app to launch/package yet. |
| Phase 7: validation | Partial | Core tests exist; OG prompt/config fixture parity, service-flow parity, Tauri command checks, React visual/smoke tests, and manual smoke docs remain. |
| Phase 8: public readiness | Deferred | Do after the app is functionally real. |

## Removal Register

These are intentionally removed, not parity gaps:

| OG area | Status | Local replacement or reason |
| --- | --- | --- |
| Accounts, users, login, signup, OIDC, MFA, sessions, password reset | Removed | Single-user local app with encrypted unlock. |
| User ownership fields and per-user uniqueness | Adapted | Collapsed to global local ownership and global uniqueness. |
| Billing, plans, subscriptions, payments, checkout, one-time purchases | Removed | BYOK local app has no project-operated billing. |
| Admin tooling, analytics dashboard, moderation queues, AI eval admin UI | Removed | No service operator/admin surface. Local token stats replace user-facing usage insight. |
| Public worlds, publication status, review submission, ratings | Removed | No public/shared content surface. Starter content is bundled locally. |
| Daily/global request limits and free-plan image limits | Removed | Provider/account limits come from the user's own API key; local app does not impose service quotas. |
| HTTP routes, websocket server, service-worker/PWA/deployment concerns | Adapted | Rebuild as typed Tauri commands and event channels; async setup/unlock/profile/settings/credential commands now exist. |
| Mongo repositories and server migrations | Adapted | Rebuild as encrypted SQLite repositories and local forward migrations. |
| Gemini provider implementation | Deferred/removed for launch | Specs target OpenAI BYOK first while keeping a provider abstraction. |
| Landing, terms, privacy, subscribe, manage subscription pages | Removed | Public website/account flows are not app surfaces. |
| Shopping-list/unfinished chat modes | Removed | Not implemented in OG product behavior and excluded by spec. |

## Shared Models And Data

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| `models/types.rs` string/id helpers | `model/strings.rs`, `model/ids.rs`, `sfid.rs` | Ported | Bounded strings and prefixed IDs are preserved. |
| `models/character.rs` | `model/character.rs` | Adapted | Core character fields, initial-message variants, creativity controls, builder session, snapshots, and image metadata exist; owner/public fields are removed. |
| `models/chat.rs` | `model/chat.rs` | Adapted | Chat, messages, sender identity, and reactions exist; user sender identity collapses to local player. |
| `models/worlds.rs` | `model/world.rs` | Adapted | World blueprints, adventures, messages, GM proposals, builder sessions, statuses, and image transform metadata exist; publication/rating/moderation fields are removed. |
| `models/profile.rs` / `models/user.rs` | `model/profile.rs`, `model/settings.rs`, `model/credentials.rs` | Adapted | Split into local app profile, player profile, settings, and encrypted provider credentials. |
| `models/ai_model.rs` | `model/ai_model.rs`, `ai/registry.rs` | Adapted | OpenAI-focused launch registry exists; OG provider sprawl is trimmed. |
| `models/websocket.rs` | `src-tauri/src/events.rs` | Adapted | Replaced by typed Tauri bridge events for chat/adventure streaming, statuses, image-ready notifications, task status, and errors. |
| `models/analytics.rs`, `models/ai_evaluation.rs` | `model/metric.rs`, `stats.rs` | Adapted/removed | User-facing token metrics exist; service-operator analytics/eval models are removed. |
| `models/device.rs`, auth/billing/payment models | none | Removed | Obsolete in the local-only app. |
| Bundled starter worlds | `seed.rs`, `model/install.rs` | Adapted | Starter seed ledger exists; verify exact OG starter payloads before UI/onboarding work. |
| Local drafts | `model/draft.rs`, `store/repo/drafts.rs` | Ported | Composer draft persistence exists for chat/adventure scopes. |
| Stored image bytes | `model/images.rs`, `store/repo/images.rs` | Adapted | Generated/uploaded images live inside the encrypted SQLite store. |

## Store And Query Behavior

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| Mongo character/chat/world repositories | `store/repo/*`, `store/schema.rs` | Adapted | Entity tables store indexed columns plus JSON record data. |
| Per-user Mongo filters | local single-user repos | Adapted | Removed by design; data belongs to the unlocked local store. |
| List paging/search | `list_characters`, `list_blueprints`, `list_adventures` | Adapted | Uses keyset pagination for database-backed lists. |
| Cascading deletes | `store/repo/characters.rs`, `store/repo/worlds.rs` | Ported | Character/world/adventure cascades are represented locally. |
| SQLCipher encrypted store | `store/db.rs`, `store/crypto.rs` | Adapted | Local-only security requirement, not OG behavior. |
| Keychain remember/forget | `keychain.rs`, `store/db.rs` | Adapted | Local convenience around encrypted unlock. |
| Schema forward migration | `store/schema.rs` | Adapted | Uses SQLite `user_version`. |
| OG-to-local fixture mapping | `tests/fixtures/og_local_models.json`, `tests/og_local_fixtures.rs` | Partial | Representative local-adapted feature records deserialize and persist through encrypted SQLite; broaden with additional OG edge-case records before Phase 3 exit. |
| Query mutation-between-pages tests | `tests/store.rs` | Untrusted | Tests exist, but the inventory has not yet verified every list against OG UI expectations. |

## AI, Prompts, And Provider Calls

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| `services/ai/openai.rs` | `ai/openai.rs`, `libs/ai-client` | Adapted | Uses the OpenAI Responses API path and BYOK key source. |
| `services/ai/gemini.rs` | none | Deferred/removed for launch | Provider abstraction remains; Gemini safety-setting behavior is not launch scope. |
| AI config/task defaults | `ai/types.rs`, `ai/registry.rs`, call-site configs | Partial | Request-level tests now pin the main chat, chat summary, character state update, builder, extraction, adventure, forced full-state update, `/gm`, and image request configs; prompt hash snapshots cover representative rendered prompts. |
| Structured JSON output and fence rescue | `ai/types.rs`, `ai/fence.rs` | Ported | Schema and lenient parse helpers exist. |
| Missing-key and transient retry behavior | `ai/service.rs` | Adapted | Local key source guards requests; retry is implemented in the service. |
| Token usage capture | `model/metric.rs`, store metrics, engines | Adapted | Local stats replace OG billing/rate-limit accounting. |
| Prompt viewer sections | `prompt/*` | Partial | Core section model exists; React prompt viewer/editor UI is missing. |

## Character Chat

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| Open/load/delete chat routes | `chat/engine.rs`, store chat repos, `src-tauri/src/commands.rs` | Partial bridge | Core can open per-character chat and store/delete chats. Tauri can open/load a character chat; delete command remains missing. |
| Streaming character reply | `chat/engine.rs`, `ai/service.rs`, `src-tauri/src/events.rs`, `src-tauri/src/commands.rs` | Partial bridge | Tauri `send_chat_message` emits player-message, chunk, completion, reaction, status, and error events around the core streamed reply. Background summary/state-update dispatch remains. |
| Prompt assembly | `prompt/character.rs`, `chat/prompts.rs` | Partial | Request-level tests pin key section ordering and config; representative OG golden payload snapshots remain. |
| Chat summary/title generation | `chat/engine.rs`, `chat/prompts.rs` | Ported | Summary and title prompts exist. |
| Reactions | `model/chat.rs`, `chat/engine.rs` | Ported | Allowed emoji filtering and persistence exist. UI missing. |
| Character state update after chat | `chat/engine.rs`, `chat/prompts.rs` | Ported | Coalesced state update path exists. Needs fixture coverage. |
| Chat list/search UI | none | Missing | React implementation required. |
| Chat composer draft restore/clear | `model/draft.rs`, store drafts | Ported core / missing UI | Core persistence exists; UI and bridge missing. |

## Characters And Character Builder

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| Manual character create/edit | `model/character.rs`, store character repos | Ported core / missing UI | Field model and persistence exist; React editor and Tauri commands missing. |
| Character list/search | `store/repo/characters.rs` | Ported core / missing UI | Keyset list exists; UI and command DTOs missing. |
| Character builder service | `character/engine.rs`, `character/prompts.rs` | Ported | Structured full-field replacement, snapshots, history, undo, and request config parity exist. |
| Character NPC extraction from worlds | `character/engine.rs`, `character/prompts.rs` | Ported | Core extraction path and request config parity exist. |
| Portrait generation/upload/clear | `image/mod.rs`, `store/repo/images.rs` | Adapted | AI generation plus local image bytes exist; upload is local-only beyond OG. |
| Portrait transform editor | none | Missing | React implementation should port OG geometry/interaction behavior. |
| Finish builder image step | `image/mod.rs` plus builder | Partial | Core image generation exists; combined builder finish command/UI missing. |

## Worlds And Adventures

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| World blueprint create/read/update/delete/list/count | `model/world.rs`, `store/repo/worlds.rs` | Ported core / missing bridge | Core model and repository exist; Tauri commands missing. |
| World builder service | `world/builder.rs` | Ported | Structured full-field replacement, snapshots, chat history, undo, metrics, and request config parity exist. |
| Adventure list/load/delete | `store/repo/worlds.rs` | Ported core / missing bridge | Core persistence exists; Tauri commands missing. |
| Adventure start and intro | `world/engine.rs`, `world/prompts.rs` | Ported | Streams intro narration and persists initial state. Needs fixture parity tests. |
| Player turn narration | `world/engine.rs`, `world/prompts.rs`, `src-tauri/src/events.rs` | Ported core / partial bridge | Streams narration through a callback; bridge event DTOs exist, but command emission is not wired yet. |
| Diff/full state updates | `world/engine.rs`, `world/state_patch.rs`, `world/response.rs` | Ported | Diff fallback to full update exists. Needs OG representative fixture tests. |
| Story memory, recent events, significant events | `world/memory.rs`, `world/prompts.rs` | Ported | Uses `story_summary` with rolling/recent sections. |
| Compaction | `world/prompts.rs` | Partial | Prompt exists; verify trigger/cadence and persistence against OG before trusting. |
| `/gm` answer/proposal flow | `world/input.rs`, `world/engine.rs`, `world/response.rs` | Ported | Classify -> answer/proposal -> accept/reject exists. UI missing. |
| Adventure-state validator | `world/state_patch.rs` | Ported but needs validation audit | Patch validator exists; add fixtures for malformed paths and schema-critical failures. |
| World cover generation/upload/clear | `image/mod.rs`, `store/repo/images.rs` | Adapted | AI generation plus local image bytes exist. |
| Cover transform editor | none | Missing | React implementation should port OG cover geometry/interaction behavior. |
| World templates | none yet | Missing | OG `manage/templates.rs` behavior should be ported or explicitly folded into starter content/editor flow. |
| Public/featured worlds, submit/withdraw review, rating | none | Removed | Obsolete account/community features. |

## UI Fidelity Inventory

The current repo has no React app yet. These OG UI surfaces are required unless marked removed:

| OG UI area | Status | Notes |
| --- | --- | --- |
| App shell, route tree, bottom nav/sidebar, titlebar | Missing | Rebuild in React/Tauri, preserving OG structure and local-only nav removals. |
| First-run/onboarding | Missing | Needs setup/unlock/provider flow plus starter worlds. |
| Unlock/setup screens | Missing | New local-native surface; must honor security specs. |
| Worlds home | Missing | Port OG list/card/search/empty/loading/error behavior, minus public/admin tabs. |
| World play screen | Missing | Port immersive layout, composer, stream status, current adventure affordances, GM proposal cards. |
| World create/edit | Missing | Port tabs, template affordances, manual editor, image selector/transform, builder entry. |
| World builder | Missing | Port chat + editable prompt/fields pattern. |
| Character list | Missing | Port cards, search, empty/loading/error states. |
| Character create/edit | Missing | Port Profile/Prompt/Initial Message/Settings tabs, image selector/transform, builder entry. |
| Character builder | Missing | Port chat + prompt tab pairing. |
| Character chat | Missing | Port bubbles, streaming feel, reactions, draft behavior, status labels. |
| Prompt viewer/editor | Missing | Port locked/editable section treatment; editable system prompts are deferred unless specs change. |
| Settings/profile | Missing | Port provider key management, theme/accent, player profile, content toggle, local-only storage actions. |
| Token stats | Missing | Build from local metrics, not OG admin analytics. |
| Modals/toasts/confirmation/error/loading/empty states | Missing | Port visible behaviors and local destructive-action confirmation patterns. |
| Admin, auth, billing, terms/privacy/landing | Removed | Not part of local app surface. |

## Asset And Styling Inventory

| OG source | Status | Notes |
| --- | --- | --- |
| `bins/soulfire-ui/input.css` | Missing in React app | Must be copied into React/Tailwind tokens as the fidelity base, not approximated. |
| `bins/soulfire-ui/src/components/{layout,buttons,modal,titlebar}.rs` | Missing in React app | Use as behavior/visual references. |
| `components/character.rs`, `components/world.rs` image renderers | Missing in React app | Port transform math and display precedence. |
| `hooks/theme.rs` | Adapted in specs / missing UI | Theme/accent model exists in local settings; React implementation pending. |
| `res/*` portraits/icons | Not imported | Decide whether to vendor OG starter assets or regenerate/localize them before UI work. |
| `manifest/ui-design-language.md` | Adapted into `specs/09-ui.md` | Continue using OG manifest and components as reference material. |

## Validation Inventory

| Validation area | Status | Next proof needed |
| --- | --- | --- |
| Core model round-trip tests | Partial | Existing tests cover local models plus a representative OG-to-local fixture for feature records; add broader OG edge-case imports. |
| Store security tests | Partial | Existing encrypted store tests exist; Tauri credential status tests prove raw keys are not returned. Re-audit logs/errors and key handling as feature commands land. |
| Cursor pagination tests | Partial | Existing tests cover keyset behavior; verify all database-backed UI lists use the cursor contract. |
| Prompt/config golden tests | Partial | Request-level config assertions cover chat, summary/state updater, builders, extraction, images, world intro/turn/diff/full update, and `/gm`; `tests/prompt_snapshots.rs` pins representative full rendered prompts with SHA-256 snapshots and anchors. Broaden with request-object snapshots as bridge/UI work exposes more DTO paths. |
| Data/service-flow fixture tests | Partial | Data fixture covers representative adapted records through serde and store persistence; core flow tests cover representative behavior. Add broader OG-derived success/failure/recovery fixtures. |
| Tauri command/event tests | Partial | `soulfire-app` has async state-boundary tests plus event serialization tests for React-facing names/fields. Add command invocation tests as feature commands/events land. |
| React smoke/visual tests | Missing | Blocked until React app exists. |
| Manual smoke checklist | Missing | Add alongside first vertical slice. |

## Next Implementation Queue

Work in this order unless the roadmap changes:

1. **Phase 3 trust pass:** broaden the current representative prompt hash snapshots with
   request-object snapshots for full engine turns once bridge DTOs exist. Request-level config
   assertions and rendered-prompt hash snapshots already cover the main chat, builder, extraction,
   world, image, state-update, and `/gm` families.
2. **Phase 3 data proof:** broaden OG-to-local model mapping fixtures beyond the current
   representative Character, Chat, ChatMessage, WorldBlueprint, Adventure, AdventureMessage,
   GmProposal, builder-session, metric, settings, profile, and draft records.
3. **Phase 4 vertical commands:** setup/unlock/settings/profile/credential and character chat
   commands exist; next expose one world adventure flow through commands/events.
4. **Phase 5 React shell:** scaffold Vite/React/Tailwind/Bun with OG tokens, custom controls, shell
   navigation, setup/unlock, and the two vertical-slice screens.
5. **Phase 5 fidelity expansion:** port the remaining editors, builders, image framing, prompt viewer,
   settings/profile, stats, and smoke/visual checks.

## Definition Of Done For Phase 2

- Each OG feature family is either mapped to local code, marked missing, or deliberately removed.
- Account/server/product fields are centralized in the removal register instead of becoming phantom
  parity tasks.
- The next implementation step is no longer vague: finish Phase 3 trust/golden coverage, then build
  the Tauri bridge and React vertical slice.
