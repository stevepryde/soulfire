# Soulfire Open Source Roadmap

Status: planning document only. This does not change the spec.

Soulfire is an open-source, local-first, BYOK port of Soulfire-OG to a desktop/mobile stack that is
easier for other people to use, inspect, fork, modify, and extend.

Soulfire-OG remains the fidelity reference. The new app should feel basically identical to the
original except for changes required by desktop/mobile, local-only storage, and the removal of
obsolete server/account scaffolding. Prompt text, prompt ordering, model settings, memory cadence,
chat/world logic, builder behavior, streaming feel, visual language, screen flows, and small
interaction details should be as close to 100% faithful as possible.

## Stack Decision

Use Tauri v2 + React for the app shell, with Rust owning the core.

Frontend stack:

- TypeScript
- React
- Tailwind CSS
- TanStack Query / TanStack Router where useful
- Vite
- Bun

Native/backend stack:

- Tauri v2
- Rust core crates for all important product logic
- SQLite for local storage
- local encryption / secure key handling
- typed Tauri commands and event channels between UI and Rust

## Why Tauri + React

If this were only a personal product, Dioxus would still be a strong choice. Soulfire-OG is already
Dioxus, the code is elegant, and a Rust-only app has a lovely simplicity. For an app built mainly
for the author, Dioxus would be defensible and maybe even preferable.

That is not the actual goal anymore.

The goal is an open-source app other people can run, trust, fork, and make their own. That changes
the priority order:

- React/TypeScript has a much larger contributor pool than Dioxus.
- More people can audit and modify a React UI without first learning Rust UI patterns.
- Tauri v2 gives one app model for desktop and mobile while keeping Rust for the sensitive core.
- Web UI tooling gives better-established accessibility, responsive layout, test, and design-system
  workflows.
- The Rust backend still carries the security, correctness, local-first, and performance advantages
  that matter most for Soulfire.
- Forks can replace or extend the UI without rewriting the core engines.

The tradeoff is real: React means the UI cannot be ported line-for-line from OG Dioxus. That is an
acceptable cost because fidelity will be enforced by specs, visual comparison, prompt/config golden
tests, and smoke tests instead of by sharing a UI framework. We do however want the UI to look
and feel almost identical to the OG UI in every way, adapted for the new single-user "native"
direction.

## Product Boundaries

Required:

- single-user local app
- BYOK provider setup
- encrypted local data
- no project-operated server
- no account system
- no billing, plans, entitlements, usage caps, admin tooling, moderation, ratings, or public content
- faithful Character Chat
- faithful Worlds/adventure engine
- faithful character/world builders
- faithful prompt internals and AI request settings
- faithful look and feel, adapted responsively to desktop and mobile
- cursor-based pagination for database-backed lists

Removed by default:

- user IDs, account ownership, auth/session/OIDC/MFA fields
- billing, subscription, plan, entitlement, quota, and price-table fields
- admin, moderation, review-queue, ratings, public/private visibility, and sharing fields
- HTTP route, websocket transport, browser/PWA, and deployment-only concepts
- multi-user uniqueness constraints where single-user global uniqueness is the real product rule

## Architecture Principle

Rust owns truth. React owns presentation.

The frontend must not become a second backend. It may hold view state, optimistic interaction state,
form drafts, and TanStack Query cache state, but durable product truth lives in Rust and SQLite.

Rust core responsibilities:

- domain models and validation
- encrypted SQLite store
- cursor pagination and query correctness
- prompt assembly
- AI request construction and streaming orchestration
- chat engine
- world/adventure engine
- memory, summaries, state patches, GM proposals
- character/world builders
- image generation/storage/framing metadata
- token metrics
- import/export if added later

Rust crate layout:

- keep product/domain logic consolidated in `soulfire-core`
- keep domain models under `soulfire_core::model`
- do not retain `sp-*` crates or personal-prefix crate names
- keep `ai-client` separate only while it remains a clean provider transport adapter; fold it into
  `soulfire-core` later if the boundary creates double handling
- do not keep a Rust markdown UI crate for the React app; use a mature React markdown renderer when
  markdown rendering is rebuilt

React responsibilities:

- app shell and navigation
- screens, forms, dialogs, cards, lists, composers, editors, and responsive layout
- local view state and ergonomic interactions
- invoking typed Tauri commands
- subscribing to Rust event channels
- rendering streamed updates without blocking the app

Bridge rules:

- no direct frontend database access
- no API keys in frontend storage
- no prompt assembly in TypeScript
- no duplicated business rules in React except lightweight client-side affordances
- commands return typed DTOs
- streaming uses event channels or equivalent Tauri primitives
- every long-running Rust operation has cancellation/backpressure semantics

## Strategy

Use a quarantine-and-rebuild approach, not incremental patching.

Keep the specs, docs, useful tests, theme assets, and any Rust core/storage code that survives audit.
Treat the current Dioxus UI, partial async edits, and previous Tauri shell as untrusted until proven
useful. Delete or replace code freely when it is cheaper than explaining every drift.

The implementation should be a faithful feature port with narrow local/native adaptations:

- Mongo repository behavior becomes encrypted SQLite repository behavior.
- Server endpoints become Tauri commands.
- Websocket/status behavior becomes Tauri event streams.
- Dioxus components become React components that match OG behavior and look/feel.
- Obsolete server/account/product subsystems are removed by default.
- Surviving feature behavior is ported faithfully after obsolete layers are stripped away.

## Phase 0: Quarantine Current State

1. Preserve the current repo state for reference.
2. Stop building on partial Dioxus async/UI edits.
3. Do not resurrect the previous minimal Tauri shell.
4. Decide which current Rust crates are trustworthy by audit, not by optimism.
5. Start the new Tauri + React work from a clean baseline.

Exit criteria:

- There is a clean working baseline for the Tauri + React rebuild.
- Existing Dioxus UI code is either archived, deleted, or clearly treated as reference-only.
- Current Rust core/storage code has an explicit keep/replace decision.

## Phase 1: Spec Update Pass

Do this before implementation changes.

1. Update the specs to describe Soulfire as an open-source, local-first desktop/mobile port of
   Soulfire-OG, with OG as the fidelity reference.
2. Change platform specs from Dioxus desktop/mobile to Tauri v2 desktop/mobile.
3. Record the frontend stack: TypeScript, React, Tailwind, TanStack, Vite, Bun.
4. Record the backend/core rule: Rust owns durable product logic and storage.
5. Add a removal/adaptation register: accounts, auth, billing, admin, public sharing, moderation,
   ratings, server transport, web/PWA concerns, Mongo-specific shapes, and per-user ownership are
   removed; SQLite, local encryption, BYOK, desktop/mobile, and token stats are deliberate changes.
6. Add a storage/query requirement that database-backed lists use cursor-based pagination, with
   stable ordering and deterministic tie-breakers.
7. Defer editable system prompts unless the spec keeps them as a compatibility-preserving overlay.
   The first parity pass should ship byte-for-byte OG defaults.
8. Update testing requirements so prompt/config parity, service-flow parity, and UI fidelity are
   first-class tests.

Exit criteria:

- The specs define the new app identity and stack clearly.
- Every deliberate deviation from OG has a requirement ID or design note.
- No spec implies Dioxus is still the target runtime.

## Phase 2: Feature Inventory And Diff

Build a feature matrix from `~/projects/app-world/soulfire`.

Inventory these areas:

- Soulfire-OG shared models, enums, validation, IDs, and serialized record shapes, explicitly
  separating current feature fields from obsolete account/server/product fields.
- `soulfire-api` chat services, adventure services, builders, images, metrics, AI provider calls,
  state validators, memory helpers, and websocket/status behavior.
- `soulfire-ui` pages, components, providers/hooks, CSS, navigation, onboarding, settings, editors,
  builders, chat, play, cards, dialogs, loading states, empty states, and error states.

Each item gets one status:

- ported without behavior change
- adapted at a named seam
- removed as obsolete local-native scaffolding
- missing
- current implementation exists but is not trusted

Exit criteria:

- The project has a concrete parity checklist.
- Removed account/server/product fields are listed once and do not keep reappearing as faux parity
  work.
- There are no vague buckets like "chat basically works" or "worlds mostly ported."

## Phase 3: Rust Core

Build or salvage the Rust core before treating the UI as real.

1. Port current feature models faithfully after deleting obsolete fields.
2. Build encrypted SQLite repositories that preserve OG logical behavior without importing Mongo
   assumptions.
3. Collapse per-user uniqueness, queries, and indexes into single-user local rules.
4. Use cursor-based pagination for every database-backed list. Cursors should be based on indexed,
   stable sort keys plus deterministic tie-breakers, never offset pagination.
5. Port prompt assembly, AI call configuration, chat, worlds, builders, image handling, metrics, and
   import/export seams into Rust.
6. Consolidate former helper/domain crates into `soulfire-core` unless a crate boundary has a clear
   public purpose.
7. Add golden prompt/config tests and service-flow fixture tests before broad UI work.
8. Re-check encrypted-at-rest behavior and key handling before trusting existing storage code.

Exit criteria:

- Current feature records round-trip through the local model layer.
- OG-to-local fixture mappings prove that removed fields are obsolete, not accidentally lost feature
  state.
- Local prompt payloads and AI configs match OG for representative fixtures.
- List queries have cursor-based pagination tests covering stable ordering, tie-breakers, and
  mutation between pages.
- Security-critical behavior is tested, not assumed.

## Phase 4: Tauri Bridge

Expose the Rust core through a typed Tauri API.

1. Define command DTOs and event payloads from the Rust core types.
2. Add commands for setup/unlock/settings, worlds, adventures, chat, builders, images, prompts, and
   stats.
3. Add event channels for streaming chat/adventure output, long-running task status, and errors.
4. Enforce command permissions/capabilities narrowly.
5. Ensure secrets never cross into frontend persistence or logs.
6. Add integration tests for command behavior where practical.

Exit criteria:

- The React app can do useful work only through typed Tauri commands/events.
- Command boundaries do not leak API keys, raw secrets, or storage internals.
- Long-running work has cancellation, timeout, and backpressure behavior.

## Phase 5: React UI Fidelity Port

Rebuild the UI in React while matching OG behavior and feel.

1. Recreate the Soulfire shell, navigation, accent system, dark theme, typography, immersive
   surfaces, and standard pages.
2. Port screens: first-run/setup, unlock, worlds home, play, chat, character list, character editor,
   world editor, builders, image framing, prompt viewer, settings, profile, and stats.
3. Use OG Dioxus components and CSS as behavioral/visual reference, not as runtime architecture.
4. Use custom dropdowns and polished app controls; do not regress to native browser selects.
5. Keep React state local to presentation and interaction; persist through Rust commands.
6. Add visual/smoke checks for every major flow.

Exit criteria:

- A user familiar with OG should recognize the app screen by screen.
- Every click target, page activation path, loading state, empty state, and error state has a smoke
  step.
- The UI is not a React-flavored redesign; it is a faithful Soulfire interface in React.

## Phase 6: Desktop And Mobile Readiness

Prove Tauri v2 works for the platforms Soulfire claims.

1. Desktop first: macOS, Windows, Linux app launch, lock/unlock, storage, AI calls, images, and
   packaging.
2. Mobile next: iOS and Android build, launch, storage location, safe-area layout, keyboard behavior,
   touch targets, image handling, and provider calls.
3. Keep platform-specific code behind narrow Rust/Tauri abstractions.
4. Document known mobile limitations honestly.
5. Set up release/package workflows only after the app is functionally real.

Exit criteria:

- Desktop builds and smoke tests pass.
- Mobile builds launch and pass basic smoke tests.
- Platform differences are documented and contained.

## Phase 7: Validation

Validate against OG and the new specs, not against hope.

Required validation:

- golden prompt/config tests for chat, worlds, builders, extraction, summaries, state updates, and
  GM commands
- service fixture tests for success, provider failure, parse failure, state-update failure, retries,
  and recovery
- data fixture tests for serialized local model compatibility and SQLite persistence
- Tauri command tests where practical
- React smoke/visual tests for all major screens and flows
- manual desktop and mobile smoke runs

Exit criteria:

- Every spec requirement has either an automated test or a documented manual smoke step.
- Every OG-derived item has one of: implemented, removed as obsolete, or explicitly deferred.
- There are no hidden "similar enough" areas.

## Phase 8: Public Readiness

Do this only after the app is real.

1. Decide whether to preserve history, squash history, or restart the public repo.
2. Audit licenses, vendored crates, binary assets, JavaScript dependencies, and generated files.
3. Re-run security review for local encryption, key storage, logs, IPC boundaries, and AI request
   payloads.
4. Clean docs so public-facing claims match verified reality.
5. Prepare contributor docs that explain the Rust core / React UI split.
6. Prepare desktop release workflows first, then mobile distribution notes.

Exit criteria:

- The repository is no longer tainted by misleading completion claims or abandoned rewrites.
- Public documentation says exactly what the app does and what remains unfinished.
- New contributors can understand where to change UI, core behavior, prompts, storage, and platform
  integrations.

## Commitment Criteria

Commit to the Tauri + React path by proving one vertical slice:

1. One character chat flow works end to end.
2. One world adventure start and one turn work end to end.
3. Prompt payloads and AI configs match OG for the chosen fixtures.
4. Rust owns storage, prompt assembly, AI orchestration, and durable state.
5. React renders a faithful Soulfire UI for those flows.
6. The app launches as a Tauri desktop app without blocking or freezing.

If the slice fails because the current repo code is too compromised, delete more of the current
implementation and continue the rebuild. Do not respond to a failed slice with another half-rewrite.
