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
| Phase 0: quarantine current state | Done | Dioxus UI was removed from this repo; the workspace now contains the Rust core crates, Tauri app crate, and React/Vite shell. |
| Phase 1: spec update pass | Done enough for implementation | Specs describe the Tauri v2 + React stack, local-only removals, cursor pagination, and React/Tauri testing expectations. |
| Phase 2: feature inventory and diff | Done here | This document is the first concrete OG parity matrix. Keep it current as work lands. |
| Phase 3: Rust core | Partial | A broad Rust core exists, including encrypted SQLite, models, chat, worlds, builders, images, metrics, pagination, request-level config coverage, representative prompt hash snapshots, and OG-to-local data fixtures. Some trust checks remain. |
| Phase 4: Tauri bridge | Partial | `src-tauri` now provides the Tauri v2 crate/config/capability scaffold, narrow app command permissions, async typed setup/unlock/profile/settings/credential commands, token-stats read/clear commands, image load/generate/upload/clear commands, character prompt-view/save-section commands, adventure prompt-view commands, character save/list/load/delete, character/world builder state/send/undo commands, NPC extraction commands, chat load/delete/open/send, world save/list/load/delete, adventure list/load/delete/start/turn/GM-proposal decision commands, and the first event DTO vocabulary. |
| Phase 5: React UI fidelity port | Partial | Vite/React/Tailwind/Bun scaffold exists with local setup/unlock, shell navigation, Tauri-backed Worlds/Characters read surfaces, and OpenAI credential status/save/delete in Settings. OG detail/editor/play screens remain reference-only until the fidelity port lands. |
| Phase 6: desktop/mobile readiness | Partial | Tauri app crate/config, command bridge, permissions, and frontend build path exist. Launch/package readiness still needs end-to-end smoke and packaging checks. |
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
| HTTP routes, websocket server, service-worker/PWA/deployment concerns | Adapted | Rebuild as typed Tauri commands and event channels; async setup/unlock/profile/settings/credential, character/chat lists and deletes, world/adventure lists and deletes, and adventure start/turn commands now exist. |
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
| Prompt viewer sections | `prompt/*`, `world/prompts.rs`, `src-tauri/src/commands/prompts.rs` | Partial bridge | Core character and adventure narration prompt section models exist. Tauri exposes character prompt view with per-section token estimates and editable authored-prompt save, plus read-only adventure next-turn prompt view with dynamic context sections. React UI remains missing. |

## Character Chat

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| Open/load/delete chat routes | `chat/engine.rs`, store chat repos, `src-tauri/src/commands/chat.rs` | Partial bridge | Core can open per-character chat and store/delete chats. Tauri can load by chat id, delete by chat id, open/load a character chat, and send a chat message. Chat list/search remains missing. |
| Streaming character reply | `chat/engine.rs`, `ai/service.rs`, `src-tauri/src/events.rs`, `src-tauri/src/commands.rs` | Partial bridge | Tauri `send_chat_message` emits player-message, chunk, completion, reaction, status, and error events around the core streamed reply. Background summary/state-update dispatch remains. |
| Prompt assembly | `prompt/character.rs`, `chat/prompts.rs` | Partial | Request-level tests pin key section ordering and config; representative OG golden payload snapshots remain. |
| Chat summary/title generation | `chat/engine.rs`, `chat/prompts.rs` | Ported | Summary and title prompts exist. |
| Reactions | `model/chat.rs`, `chat/engine.rs` | Ported | Allowed emoji filtering and persistence exist. UI missing. |
| Character state update after chat | `chat/engine.rs`, `chat/prompts.rs` | Ported | Coalesced state update path exists. Needs fixture coverage. |
| Chat list/search UI | none | Missing | React implementation required. |
| Chat composer draft restore/clear | `model/draft.rs`, store drafts, `src-tauri/src/commands/drafts.rs` | Partial bridge / missing UI | Core persistence exists; Tauri exposes get/save/clear chat draft commands. Composer UI remains missing. |

## Characters And Character Builder

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| Manual character create/edit | `model/character.rs`, store character repos, `src-tauri/src/commands/characters.rs` | Partial bridge / missing UI | Field model and persistence exist. Tauri exposes save/list/load/delete commands and clamps creativity on save. React editor UI remains missing. |
| Character list/search | `store/repo/characters.rs`, `src-tauri/src/commands/characters.rs` | Partial bridge / missing UI | Keyset list exists and Tauri exposes cursor list/load/delete commands. React list UI remains missing. |
| Character builder service | `character/engine.rs`, `character/prompts.rs`, `src-tauri/src/commands/builders.rs` | Partial bridge | Structured full-field replacement, snapshots, history, undo, and request config parity exist. Tauri exposes builder state/send/undo; React UI remains missing. |
| Character NPC extraction from worlds | `character/engine.rs`, `character/prompts.rs`, `src-tauri/src/commands/builders.rs` | Partial bridge | Core extraction path and request config parity exist. Tauri exposes extraction with task status and character-ready event; React UI remains missing. |
| Portrait generation/upload/clear | `image/mod.rs`, `store/repo/images.rs`, `src-tauri/src/commands/images.rs` | Partial bridge | AI generation plus local image bytes exist; upload is local-only beyond OG. Tauri exposes byte load, generate/regenerate, upload, clear, and image-ready events; React UI remains missing. |
| Portrait transform editor | none | Missing | React implementation should port OG geometry/interaction behavior. |
| Finish builder image step | `image/mod.rs` plus builder | Partial | Core image generation and image bridge commands exist; combined builder finish command/UI missing. |

## Worlds And Adventures

| OG source | Local source | Status | Notes |
| --- | --- | --- | --- |
| World blueprint create/read/update/delete/list/count | `model/world.rs`, `store/repo/worlds.rs`, `src-tauri/src/commands/worlds.rs` | Partial bridge | Core model and repository exist. Tauri exposes save, cursor list, load, and delete commands; count command and UI remain missing. |
| World builder service | `world/builder.rs`, `src-tauri/src/commands/builders.rs` | Partial bridge | Structured full-field replacement, snapshots, chat history, undo, metrics, and request config parity exist. Tauri exposes builder state/send/undo; React UI remains missing. |
| Adventure list/load/delete | `store/repo/worlds.rs`, `src-tauri/src/commands/worlds.rs` | Partial bridge | Core persistence exists. Tauri exposes cursor list, in-progress list, load-with-messages/pending-proposals, and delete commands. React UI remains missing. |
| Adventure start and intro | `world/engine.rs`, `world/prompts.rs`, `src-tauri/src/commands.rs` | Partial bridge | Streams intro narration and persists initial state. Tauri `start_adventure` creates from a blueprint and emits intro completion/status events. Needs fixture parity tests. |
| Player turn narration | `world/engine.rs`, `world/prompts.rs`, `src-tauri/src/events.rs`, `src-tauri/src/commands.rs` | Partial bridge | Tauri `take_adventure_turn` emits user action echo, narration chunks, narration completion, ready-status, task-status, and error events around the core streamed turn. |
| Diff/full state updates | `world/engine.rs`, `world/state_patch.rs`, `world/response.rs` | Ported | Diff fallback to full update exists. Needs OG representative fixture tests. |
| Story memory, recent events, significant events | `world/memory.rs`, `world/prompts.rs` | Ported | Uses `story_summary` with rolling/recent sections. |
| Compaction | `world/prompts.rs` | Partial | Prompt exists; verify trigger/cadence and persistence against OG before trusting. |
| `/gm` answer/proposal flow | `world/input.rs`, `world/engine.rs`, `world/response.rs`, `src-tauri/src/commands/adventure.rs` | Partial bridge | Classify -> answer/proposal -> accept/reject exists. Tauri turn command emits command echo/completion and proposal-ready events; accept/reject commands return the decided proposal, updated adventure, and remaining pending proposals. UI remains missing. |
| Adventure-state validator | `world/state_patch.rs` | Ported but needs validation audit | Patch validator exists; add fixtures for malformed paths and schema-critical failures. |
| World cover generation/upload/clear | `image/mod.rs`, `store/repo/images.rs`, `src-tauri/src/commands/images.rs` | Partial bridge | AI generation plus local image bytes exist. Tauri exposes byte load, generate/regenerate, upload, clear, and image-ready events; React UI remains missing. |
| Cover transform editor | none | Missing | React implementation should port OG cover geometry/interaction behavior. |
| World templates | none yet | Missing | OG `manage/templates.rs` behavior should be ported or explicitly folded into starter content/editor flow. |
| Public/featured worlds, submit/withdraw review, rating | none | Removed | Obsolete account/community features. |

## UI Fidelity Inventory

The current repo has a React shell only. These OG UI surfaces are required unless marked removed:

| OG UI area | Status | Notes |
| --- | --- | --- |
| App shell, route tree, bottom nav/sidebar, titlebar | Partial | React shell, titlebar, desktop sidebar, and mobile bottom nav exist. Route/detail surfaces remain to be wired. |
| First-run/onboarding | Partial | Setup/unlock scaffold exists. Provider key flow, starter worlds, and first-run content affordances remain. |
| Unlock/setup screens | Partial | Local-native setup/unlock calls the async Tauri store commands and works in browser preview fallback. Password/security UX still needs full smoke coverage. |
| Worlds home | Partial data-backed shell | React loads in-progress adventures plus searchable, cursor-paged world blueprints through Tauri commands with loading/error/empty states and read-only blueprint detail panels. Port OG card richness, create, and full editor/play entry behavior, minus public/admin tabs. |
| World play screen | Missing | Port immersive layout, composer, stream status, current adventure affordances, GM proposal cards. Tauri draft bridge for adventure composers exists; React can inspect active adventure state and next-turn prompt sections from the shell, but play UI remains missing. |
| World create/edit | Missing | Port tabs, template affordances, manual editor, image selector/transform, builder entry. |
| World builder | Missing | Port chat + editable prompt/fields pattern. |
| Character list | Partial data-backed shell | React loads/searches cursor-paged saved characters through Tauri commands with loading/error/empty states and read-only character detail panels. Port OG card art, create, editor, and chat entry behavior. |
| Character create/edit | Missing | Port Profile/Prompt/Initial Message/Settings tabs, image selector/transform, builder entry. |
| Character builder | Missing | Port chat + prompt tab pairing. |
| Character chat | Missing | Port bubbles, streaming feel, reactions, draft behavior, status labels. |
| Prompt viewer/editor | Partial data-backed shell | Character and adventure prompt-view commands exist. React shows character and adventure prompt sections with locked/editable labels and token estimates from shell detail panels, and saves editable character prompt sections back through Tauri. Adventure prompt UI remains read-only as current-adventure changes go through `/gm`. |
| Settings/profile | Partial data-backed shell | Store status/schema/runtime, lock-store action, OpenAI credential status/save/delete, accent swatches, adult-content toggle editing, and player profile editing exist. Port remaining local-only storage actions and OG settings affordances. |
| Token stats | Partial data-backed shell | Tauri exposes aggregate, per-chat, per-adventure, and clear-history token stats commands built from local metrics. React exposes aggregate totals, model/operation breakdowns, and in-app clear-history confirmation; scoped chat/adventure stats remain. |
| Modals/toasts/confirmation/error/loading/empty states | Partial | Loading/error/empty states exist on shell panels, and token-history clearing uses an in-app confirmation modal. Port remaining OG modal/toast patterns. |
| Admin, auth, billing, terms/privacy/landing | Removed | Not part of local app surface. |

## Asset And Styling Inventory

| OG source | Status | Notes |
| --- | --- | --- |
| `bins/soulfire-ui/input.css` | Partial in React app | React shell has initial Tailwind/CSS tokens. Continue porting OG CSS tokens and component behavior as the fidelity base. |
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
| Tauri command/event tests | Partial | `soulfire-app` has async state-boundary tests plus event serialization tests for React-facing names/fields, and core observed-progress tests for chat/adventure echoes. Command registration and permission schemas are generated by the app crate checks; add direct command invocation tests as feature commands/events land. |
| React smoke/visual tests | Missing | React app exists; add browser/visual smoke checks with the first data-backed vertical slice. |
| Manual smoke checklist | Partial | `docs/MANUAL_SMOKE.md` covers the current Tauri/React shell and bridge surfaces; extend as editors, chat, play, and builders land. |

## Next Implementation Queue

Work in this order unless the roadmap changes:

1. **Phase 3 trust pass:** broaden the current representative prompt hash snapshots with
   request-object snapshots for full engine turns once bridge DTOs exist. Request-level config
   assertions and rendered-prompt hash snapshots already cover the main chat, builder, extraction,
   world, image, state-update, and `/gm` families.
2. **Phase 3 data proof:** broaden OG-to-local model mapping fixtures beyond the current
   representative Character, Chat, ChatMessage, WorldBlueprint, Adventure, AdventureMessage,
   GmProposal, builder-session, metric, settings, profile, and draft records.
3. **Phase 4 command breadth:** setup/unlock/settings/profile/credential, stats,
   character save/list-load-delete, chat load-delete/open-send, world save/list-load-delete,
   adventure list-load-delete/start-turn, image load-generate-upload-clear, character/world builder,
   NPC extraction, character/adventure prompt-view, and GM proposal accept/reject commands exist; next
   add direct command invocation tests where command behavior is not already covered by core or event
   serialization tests.
4. **Phase 5 fidelity expansion:** port the remaining editors, builders, image framing, prompt viewer,
   settings/profile, stats, and smoke/visual checks.

## Definition Of Done For Phase 2

- Each OG feature family is either mapped to local code, marked missing, or deliberately removed.
- Account/server/product fields are centralized in the removal register instead of becoming phantom
  parity tasks.
- The next implementation step is no longer vague: finish Phase 3 trust/golden coverage, then build
  the Tauri bridge and React vertical slice.
