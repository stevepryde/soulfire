# Soulfire OG Parity Roadmap

Status: planning document only. This does not change the spec.

This roadmap replaces prior planning notes until every claimed layer has been re-audited against
Soulfire-OG. The goal is not a broadly similar local app. The goal is a faithful native/local
Soulfire that preserves OG behavior, feel, prompts, and feature workflows while intentionally
removing obsolete server-product scaffolding:

- SQLite/encrypted local storage instead of Mongo-backed server storage.
- Dioxus desktop/mobile instead of Dioxus web.
- Single-user local BYOK instead of accounts, billing, admin, public content, or moderation.
- No user/account ownership fields, visibility/publication fields, plan/entitlement fields, admin
  fields, moderation fields, or web/backend transport fields unless a current local feature actually
  needs them.

## Recommendation

Stay with Dioxus for the parity rewrite.

Tauri + React is a good general-purpose desktop stack, but it is the wrong default for this phase.
Soulfire-OG is already Rust + Dioxus web, so Dioxus gives the project a real mechanical-port path:
models, services, prompts, component structure, styling, and UI state can be copied or adapted with
minimal semantic translation. React would force a UI rewrite while the backend is also being
repaired, which increases correctness risk exactly where the project needs less interpretation.

Dioxus still has real risk on desktop/mobile: renderer blocking, async ownership, hook-order issues,
and thinner ecosystem coverage (although that point matters less because the original is already
using dioxus). The mitigation is architectural, not a stack pivot: Dioxus must be
the renderer, while storage, AI, streaming, and long-running native work live behind an async
coordinator with explicit backpressure. If a scoped parity slice cannot be made responsive on
desktop/mobile, Tauri + React becomes the fallback. It should not be the starting point.
If a switch to Tauri + React is needed, confirm explicitly with the user beforehand.

## Working Principle

Treat Soulfire-OG as the behavioral reference, not as schema archaeology.

The app should feel and behave like OG where the feature still exists. It should not preserve
obsolete account-era or web-server-era machinery just because OG had it. A feature is complete only
when all relevant current-product surfaces match OG, or when a spec-backed local/native decision
explains the difference:

- data shape and validation, after removing obsolete ownership/product fields
- prompt text, prompt ordering, model settings, reasoning settings, and output limits
- service control flow, memory cadence, summary cadence, and failure behavior
- streaming/status behavior
- UI layout, affordances, states, copy, and interaction feel
- tests or manual smoke steps proving the contract

Default removal list:

- user IDs, account/profile ownership, auth/session/OIDC/MFA fields
- billing, subscription, plan, entitlement, usage-cap, and price-table fields
- admin, moderation, review-queue, ratings, public/private visibility, and sharing fields
- HTTP route, websocket transport, browser/PWA, and deployment-only concepts
- multi-user uniqueness constraints where single-user global uniqueness is the real product rule

## Strategy

Use a quarantine-and-rebuild approach, not incremental patching.

Do not try to polish the current app into parity screen by screen. Keep assets, specs, test
infrastructure, and any proven security/storage work, but treat current app screens and service
engines as untrusted until they pass an OG comparison. Replace modules when replacement is cleaner
than explaining every drift.

The intended result is not a greenfield invention. It is a mechanical rewrite with narrow local
adaptations:

- Mongo repository calls become encrypted SQLite repository calls.
- HTTP endpoints and websocket events become in-process commands and event streams.
- Web-only browser APIs become native desktop/mobile equivalents, if needed at all.
- Server/account/product subsystems are removed by default because they are obsolete in the new app.
- Surviving features are ported faithfully after those obsolete layers are stripped away.

## Phase 0: Quarantine The Current State

1. Preserve the repo state and dirty work for reference, but stop building on partial conversion code.
2. Start the parity rewrite from a clean branch or worktree.
3. Mark the current implementation as suspect until each module passes an OG comparison.
4. Keep only low-risk assets immediately: specs, CSS/theme assets, packaging notes, and useful tests.
5. Re-audit storage/security before keeping it; do not assume it is correct because it compiles.

Exit criteria:

- There is a clean working baseline for the parity rewrite.
- Dirty partial async/UI edits are either reverted, parked in a branch, or deliberately reapplied.
- Removed planning notes are no longer treated as evidence of completion.

## Phase 1: Spec Update Pass

Do this before implementation changes.

1. Update the specs to make native/local parity operational: every feature should say what behavior
   must be preserved, what platform/storage seam changes, and what obsolete OG scaffolding is gone.
2. Add a removal/adaptation register: accounts, auth, billing, admin, public sharing, moderation,
   ratings, server transport, web/PWA concerns, Mongo-specific shapes, and per-user ownership are
   removed; SQLite, local encryption, BYOK, desktop/mobile, and token stats are deliberate changes.
3. Add a storage/query requirement that database-backed lists use cursor-based pagination, with
   stable ordering and deterministic tie-breakers, so large local stores stay efficient and page
   boundaries remain correct as rows are inserted or updated.
4. Defer editable system prompts unless the spec keeps them as a compatibility-preserving overlay.
   The first parity pass should ship byte-for-byte OG defaults.
5. Update testing requirements so prompt/config parity and service-flow parity are first-class tests.
6. Update UI requirements so "faithful look and feel" means component-level parity, not only theme
   similarity.

Exit criteria:

- The specs define a 1:1 parity target strongly enough that code can be judged against them.
- Every deliberate deviation from OG has a requirement ID or design note.

## Phase 2: Feature Inventory And Diff

Build a feature matrix from `~/projects/app-world/soulfire`.

Inventory these areas:

- `lib-soulfire` shared models, enums, validation, IDs, and serialized record shapes, explicitly
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
- There are no vague buckets like "chat basically works" or "worlds mostly ported."
- Removed account/server/product fields are listed once and do not keep reappearing as faux parity
  work.

## Phase 3: Data Model And SQLite Adapter

Port OG feature models faithfully after deleting obsolete fields.

1. Copy shared data structures, enum values, defaults, validation bounds, and serde shapes for
   current features.
2. Build SQLite repositories that preserve OG logical behavior without importing Mongo assumptions.
3. Delete user/account ownership, visibility, billing, admin, moderation, rating, public-sharing,
   and transport-only fields from the local model.
4. Collapse per-user uniqueness, queries, and indexes into single-user local rules.
5. Use cursor-based pagination for every database-backed list. Cursors should be based on indexed,
   stable sort keys plus a deterministic tie-breaker, never offset pagination, so pagination remains
   efficient and correct under inserts, updates, and large local datasets.
6. Add golden serialization fixtures for local core records, with explicit mapping notes from OG
   records where fields were removed.
7. Re-check encrypted-at-rest behavior and key handling before trusting existing storage code.

Exit criteria:

- Current feature records round-trip through the local model layer.
- OG-to-local fixture mappings prove that removed fields are obsolete, not accidentally lost feature
  state.
- SQLite storage is a boring adapter, not a behavior rewrite.
- List queries have cursor-based pagination tests covering stable ordering, tie-breakers, and
  mutation between pages.
- Security-critical behavior is tested, not assumed.

## Phase 4: Backend Mechanical Port

Port service logic from OG before improving structure.

Priority order:

1. AI request construction: model selection, reasoning effort, cache behavior, temperatures, token
   limits, structured output handling, streaming, retries, and error mapping.
2. Character chat: opening message, send flow, reactions, summary cadence, character-state updater,
   title updates, message normalization, and status events.
3. Worlds: adventure start, turn engine, memory ladder, state patch/full replacement, validation,
   stale-state healing, completion handling, `/gm` answer/proposal flow, accept/reject, and NPC
   extraction.
4. Character and world builders: prompts, replacement semantics, undo/snapshot behavior, validation,
   and failure preservation.
5. Images: generation prompts, regenerate/clear behavior, transform storage, and upload behavior if
   retained as a local-only addition.
6. Metrics/token stats as local additions that do not alter OG flow.

Implementation rule:

- Copy feature behavior first. Strip obsolete account/server/product scaffolding as part of the port.
  Refactor only after parity tests exist.
- Any changed control flow needs a written reason tied to a platform/storage seam.
- Prompt text and call-site configs should be golden-tested.

Exit criteria:

- For representative inputs, local prompt payloads and AI configs match OG or have a spec-backed
  difference.
- Service tests prove the important success and failure flows.
- No backend work blocks the UI thread.

## Phase 5: UI Mechanical Port

Copy the OG Dioxus UI shape before redesigning anything.

1. Keep OG CSS/theme assets as the baseline.
2. Port components and pages in the same conceptual structure: app shell, navigation, home/worlds,
   play, chat, character editor, world editor, builders, settings/profile, dialogs, cards, image
   transform editors, loading/error/empty states.
3. Replace web providers/hooks with native-compatible Dioxus state and event streams.
4. Remove account/admin/billing/public/moderation/rating affordances because they are not part of the
   local product.
5. Avoid introducing new UI patterns during parity.

Exit criteria:

- A user familiar with OG should recognize the app screen by screen.
- Every click target, page activation path, and loading/error state has a smoke step.
- Screens are not approximations with copied colors; they are component-level ports.

## Phase 6: Native Async Hardening

Make the desktop/mobile architecture explicit.

1. Dioxus renders state and sends user intents.
2. A backend coordinator owns storage, AI calls, streaming, image work, and long-running tasks.
3. UI state receives snapshots/events; it does not run blocking storage or network work in render.
4. Every long operation has timeout/cancellation/backpressure semantics.
5. Streaming updates are batched enough to keep the renderer responsive.

Exit criteria:

- No synchronous database, filesystem, image, or network work runs on the UI thread.
- Repeated clicks cannot spawn unbounded duplicate work.
- Long-running tasks shut down cleanly.
- Desktop smoke testing shows no CPU spin or frozen renderer.

## Phase 7: Parity Validation

Validate against OG, not against hope.

Required validation:

- Golden prompt/config tests for chat, worlds, builders, extraction, summaries, state updates, and
  GM commands.
- Service fixture tests for success, provider failure, parse failure, state-update failure, retries,
  and recovery.
- Data fixture tests for serialized model compatibility and SQLite persistence.
- Manual desktop smoke tests for all major screens and flows.
- Mobile build/smoke checks once desktop parity is stable.

Exit criteria:

- Every spec requirement has either an automated test or a documented manual smoke step.
- Every OG-derived item has one of: implemented, removed as obsolete, or explicitly deferred.
- There are no hidden "similar enough" areas.

## Phase 8: Public Readiness

Do this only after parity is real.

1. Decide whether to preserve history, squash history, or restart the public repo.
2. Audit licenses, vendored crates, binary assets, and third-party code.
3. Re-run security review for local encryption, key storage, logs, and AI request payloads.
4. Clean docs so public-facing claims match verified reality.
5. Prepare packaging/release flows for desktop first, then mobile.

Exit criteria:

- The repository is no longer tainted by misleading completion claims or abandoned rewrites.
- Public documentation says exactly what the app does and what remains unfinished.

## Commitment Criteria

Commit to this path only if the first parity slice succeeds:

1. One character chat flow is copied from OG end to end.
2. One world adventure start and one turn are copied from OG end to end.
3. Prompt payloads and AI configs match OG for the chosen fixtures.
4. The Dioxus desktop UI remains responsive while streaming.
5. The UI looks and behaves recognizably like OG for those flows.

If that slice fails because Dioxus desktop/mobile cannot be made stable, then revisit Tauri + React.
If it fails because the current repo code is too compromised, delete more of the current
implementation and continue the mechanical port. Do not respond to a failed slice with another
half-rewrite.
