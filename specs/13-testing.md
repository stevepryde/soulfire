# Testing

**Purpose:** how tests **validate the code against the spec** for this single-user, local, BYOK
desktop/mobile app — the traceability rule, the testability seams the code must expose, what must be
covered, and what is automated versus manual. Strategy and outcomes only; harness/tooling specifics
live in Design notes. See [`../AGENTS.md`](../AGENTS.md) for the spec → code → tests loop and
[SPEC_GUIDE.md](SPEC_GUIDE.md) for altitude.

Prefix: `TEST`.

## Requirements

- **TEST-1 Tests validate the spec.** Every normative requirement is covered by an automated test or
  a documented manual smoke step, and each test **names the requirement ID(s) it proves**. Coverage is
  traceable both ways: every requirement → a test, and every test → the requirement it validates.
- **TEST-2 Rust-first, unit-test-led.** The core — store, AI orchestration, prompt assembly, the turn
  engine, the memory ladder, the state validator, builders, summarizers, metering — is plain Rust
  exercised by the project's Rust harness, with **pure logic covered by unit tests** and multi-step
  flows covered by integration tests. The product is a native desktop/mobile app, not a web service;
  no browser/JavaScript E2E stack is introduced.
- **TEST-3 Observable, deterministic, isolated.** Every test asserts **observable** behavior at a
  boundary (a returned result, persisted on-disk state, an emitted event/stream, an error condition,
  a timing/limit) and is **deterministic**: the same inputs produce the same pass/fail every run.
  Tests use **ephemeral, isolated state** — a temporary data directory and a throwaway encrypted store
  created per run and torn down afterward — and an **injected clock** so time-dependent behavior
  (timeouts, cadences, stale-lock expiry, timestamps) is reproducible. Runs never collide and leave
  nothing behind.
- **TEST-4 No live external dependencies.** Tests run with **no network access and no live external
  service**. The only external dependency in the product — the user's AI provider — is exercised
  through a **substitutable boundary** (TEST-5) backed by a deterministic in-process implementation;
  no test contacts a real provider or any other network endpoint.
- **TEST-5 Testability seams the code must expose.** The following are **substitutable boundaries** so
  orchestration is testable deterministically and offline: the **AI provider** (text, streamed text,
  structured-JSON, and image generation, per `AI`), the **secure credential/keychain store** (per
  `SEC-7`), and the **clock** (TEST-3). The substituted AI provider returns canned, deterministic
  responses (including scripted streaming deltas, structured-JSON payloads, errors, and timeouts) and
  **records the exact request payloads** so a test can assert what was sent. Feature code depends on
  these boundaries, never on a concrete provider/clock/keychain directly.
- **TEST-6 UI logic is extracted and unit-tested; the visual contract is smoke-tested.** Logic that
  would otherwise live inside view components — input parsing (e.g. `/gm` vs action vs unknown slash,
  `WORLD-15`), draft handling, status/label derivation, streaming-buffer assembly, markdown/reaction
  rendering decisions, list/pagination state — is factored into **pure, unit-testable functions** kept
  out of the component tree and tested directly. The **visual/interaction contract** in `UI` (layout,
  theming, immersive vs standard chrome, responsiveness, touch targets) is validated by a **documented
  manual smoke checklist** per platform, because no mature automated end-to-end harness exists for the
  native UI; optional lightweight component-render smoke tests may supplement but do not replace it.
- **TEST-7 Data & store coverage.** Automated tests cover: round-trip persistence of every entity with
  fields intact; ID prefix correctness; validation and clamping on save (creativity controls, length
  bounds); reaction filtering to the allowed emoji set; singleton enforcement (one app profile, one
  player profile, one settings row); and referential integrity / cascade deletes leaving no orphans.
  (Validates [data-model.md](01-data-model.md) DATA-1..23.)
- **TEST-8 Security coverage.** Automated tests prove at-rest protection and unlock: stored content and
  credentials are **unreadable on disk without the key**; opening data requires the correct master
  password; a wrong password leaves the store locked, intact, and inaccessible; re-key makes the old
  password fail and the new one succeed; the device-remembered unlock (via the substituted keychain,
  TEST-5) unlocks without a prompt and is removed when disabled; and an API key never appears
  unmasked in any error, log line, or returned value. (Validates [storage-security.md](02-storage-security.md)
  SEC-1..12.)
- **TEST-9 AI layer coverage.** Against the substituted provider (TEST-5): a missing key for the
  required provider produces an actionable "add your API key" condition and sends nothing; structured
  calls return schema-conforming objects and a fenced ```` ```json ```` response parses; model
  selection precedence (entity → profile default → registry default) resolves and persists; streaming
  delivers deltas then finalizes, a no-first-token stall surfaces an error and saves nothing, and a
  mid-stream stall keeps the partial text; transient errors retry then resolve while non-transient
  errors fail cleanly and leave state intact; and each metered call writes exactly the expected
  usage entries (and a zero/zero call writes none). (Validates [ai-integration.md](03-ai-integration.md)
  AI-1..16.)
- **TEST-10 Prompt coverage.** Automated tests cover prompt assembly as the OG-fidelity contract: the
  named sections appear in the fixed order, with optional sections present/absent per entity state;
  each section's locked/editable classification is correct; content toggles (incl. **adult content**)
  add/remove exactly their delimited sub-sections and never remove a required section; the assembled
  prompt is captured by a **snapshot/golden test** so any change is visible and intentional; and an
  edit made through the prompt viewer changes the same backing field as the normal editor (one source
  of truth). (Validates [system-prompts.md](04-system-prompts.md) PROMPT-1..12.)
- **TEST-11 Chat coverage.** Automated tests cover: verbatim vs generated opening message; a sent
  message persists, streams a reply, and finalizes; a trailing allowed-emoji is extracted into a
  reaction; the rolling summary regenerates on cadence and resets its counter and a failed summary
  preserves the prior one; and the character-state updater runs for world-extracted characters and is
  **serialized/coalesced per character** so two updates never interleave. (Validates [chat.md](05-chat.md)
  CHAT-1..14.)
- **TEST-12 Character coverage.** Automated tests cover: manual-editor validation and clamping;
  builder turns that apply full-replacement fields, push a snapshot, and support undo (disabled with
  no snapshots; caps enforced); and NPC extraction producing a character with non-empty
  `extracted_context` and `character_state`, recorded origin, and an opened chat, with a failed
  extraction leaving nothing partial. (Validates [characters.md](06-characters.md) CHAR-1..13.)
- **TEST-13 Worlds coverage.** Automated tests cover the turn engine and state: a turn echoes the
  action, then streams and persists narration, then updates state, with **state-update failure
  non-fatal** (narration kept, state unchanged); the per-adventure single-flight lock refuses a
  concurrent turn and self-heals after the stale-lock expiry; the adventure-state initializes with the
  required sections and excludes not-yet-known content; the memory ladder honors its caps (recent ≤20,
  significant ≤30 with weighted pruning and stable ids, story summary keeps both sections) and the
  no-wipe guard; diff-first updates apply set/append/remove patches with no partial commit on failure
  and fall back to a full update on a bad path or after the diff threshold; the state validator rejects
  malformed paths / out-of-range indices / non-object roots; and `/gm` classify → answer/proposal →
  diff → **accept/reject** stages changes without committing until accepted, altering only the
  adventure's private blueprint copy. (Validates [worlds.md](07-worlds.md) WORLD-1..21.)
- **TEST-14 Images coverage.** Against the substituted provider (TEST-5): generation runs without
  blocking, renders on success, and leaves the prior image on failure; regeneration bumps the version;
  stored/uploaded image bytes are **unreadable on disk without the key**; the transform persists; and
  rendering precedence (stored image → emoji → default) holds. (Validates [images.md](08-images.md)
  IMG-1..8.)
- **TEST-15 Token-statistics coverage.** Automated tests cover: one usage entry per metered call with
  correct label, model id, and **separate** input / cached-input / output counts (no double-counting);
  aggregates that equal the sum of entries and partition by model and operation; clearing history;
  per-chat/per-adventure rollups reconciling with the aggregate; and that **no cost figure** appears
  anywhere. (Validates [token-stats.md](11-token-stats.md) STAT-1..6.)
- **TEST-16 Onboarding coverage.** Automated tests cover: first launch requires setting a master
  password and adding a key before any AI action; starter worlds seed as editable blueprints on first
  launch and a deleted starter does **not** resurrect or duplicate on relaunch; and the returning-user
  landing prioritizes "continue" when an in-progress adventure exists. The first-run story launch and
  name-capture screens are covered by their extracted logic (TEST-6) plus a manual smoke step.
  (Validates [onboarding.md](10-onboarding.md) ONB-1..7.)
- **TEST-17 App smoke coverage.** A small end-to-end smoke exercises the two core journeys — start an
  adventure → take a turn → resume, and start a chat → send a message → react — through the same
  in-process layer the UI drives, with the substituted provider serving all model/image calls and no
  live network. Where a step is not yet driveable end to end, a documented manual smoke step stands in
  until it is, satisfying TEST-1.

## Acceptance criteria

- Every normative requirement in every spec maps to at least one row in the traceability table below
  or to a documented manual smoke step, and every listed test names the requirement ID(s) it proves
  (TEST-1).
- The full automated suite runs to completion with networking disabled and no external credentials,
  uses only per-run temporary state, and leaves nothing behind (TEST-3, TEST-4).
- Running the suite twice produces identical pass/fail results, and snapshot/golden tests (assembled
  prompts) and byte-comparison tests (encrypted-at-rest, image bytes) match exactly both times
  (TEST-3, TEST-10).
- Inspecting the substituted provider's recorded payloads in a test shows exactly what the spec says
  is sent, and no test makes a network call (TEST-4, TEST-5).
- A test confirms the store and credentials are unreadable on disk without the key, that a wrong
  master password leaves data locked and intact, and that re-key and device-remembered unlock behave
  as specified (TEST-8).
- A test confirms a `/gm` change is staged with an accept/reject diff and commits nothing until
  accepted, and that a forced state-update failure preserves the narration and leaves state unchanged
  (TEST-13).

## Acceptance traceability

Index from requirement areas to their validating tests. Keep it updated as part of every change
(spec → test). Until implementation begins it lists **intended** coverage; fill in concrete test names
as they land. Each landed test names, in its body or name, the requirement ID(s) it proves (TEST-1).

| Area | Requirements | Validated by |
|------|--------------|--------------|
| Data model & store | [01-data-model.md](01-data-model.md) DATA-1..23 | store unit + integration tests (round-trip, id prefixes, validation/clamping, reaction filtering, singletons, cascade-delete integrity) |
| Storage & security | [02-storage-security.md](02-storage-security.md) SEC-1..12 | encryption + unlock tests (ciphertext-on-disk, password required, wrong-password lock, re-key, substituted-keychain remember/remove, key-never-leaks) |
| AI integration | [03-ai-integration.md](03-ai-integration.md) AI-1..16 | AI-layer tests vs substituted provider (missing-key, structured + fence parse, model precedence/persist, streaming + idle-timeout, retry/error, metering) |
| System prompts | [04-system-prompts.md](04-system-prompts.md) PROMPT-1..12 | prompt assembly unit tests + golden snapshots (section order, locked/editable, toggles incl. adult content, viewer↔editor one-source) |
| Character chat | [05-chat.md](05-chat.md) CHAT-1..14 | chat flow tests (opening, stream, reaction extraction, summary cadence/reset, per-character coalesced state update) |
| Characters | [06-characters.md](06-characters.md) CHAR-1..13 | editor + builder tests (validation/clamp, full-replacement apply, snapshot/undo, NPC extraction, failure leaves nothing) |
| Worlds | [07-worlds.md](07-worlds.md) WORLD-1..21 | turn-engine + state tests (turn ordering, non-fatal state update, single-flight lock + stale-heal, state schema init, memory caps/pruning/no-wipe, diff-first + full fallback, validator rejects, `/gm` stage/accept/reject) |
| Images | [08-images.md](08-images.md) IMG-1..8 | image tests vs substituted provider (async gen, failure keeps prior, version bump, ciphertext bytes, transform persist, precedence) |
| UI logic & contract | [09-ui.md](09-ui.md) UI-1..23 | extracted view-logic unit tests (input parse, status/label, streaming buffer, drafts) + manual smoke checklist (`docs/MANUAL_SMOKE.md`) |
| Onboarding | [10-onboarding.md](10-onboarding.md) ONB-1..7 | onboarding tests (setup gates AI, starter seed no-dup/no-resurrect, returning-user continue) + manual smoke |
| Token statistics | [11-token-stats.md](11-token-stats.md) STAT-1..6 | metering aggregation tests (per-call entry, separate input/cached/output, aggregates partition, clear, rollup reconcile, no cost) |
| Platform & packaging | [12-platform-packaging.md](12-platform-packaging.md) PKG-1..6 | build/launch checks per target (CI/manual), data-location + forward-migration tests, license-presence check |
| Core journeys | cross-cutting | app smoke (adventure: start→turn→resume; chat: start→send→react) via in-process layer + substituted provider |

## Design notes (non-normative)

- **Harness.** Standard Rust tests: inline `#[cfg(test)]` modules for pure logic (prompt assembly,
  input parsing, memory-ladder caps/pruning, state-patch path parsing, numeric/validation clamps,
  aggregation) and `tests/` integration binaries for store, AI-orchestration, chat/adventure flows.
  Each integration test allocates a fresh temp data directory (e.g. `tempfile`) and an isolated
  encrypted store and tears them down on drop (TEST-3).
- **Seams (TEST-5) as traits.** Express the AI provider, secure-credential store, and clock as Rust
  traits the core depends on; production wires the real OpenAI adapter / OS keychain / system clock,
  tests wire deterministic fakes. The fake provider returns scripted responses and **records request
  payloads** for assertion; the fake clock advances on command so timeouts, summary cadence
  (every-N), stale-lock expiry, and timestamps are exact. This trait-seam design is the primary
  testability lever and should be in place from the first orchestration code.
- **Golden prompts (TEST-10).** Snapshot the assembled prompt per scenario (plain character,
  world-linked character with dynamic state, adventure narration, each toggle on/off) so OG-fidelity
  drift is caught in review; update snapshots only when a `PROMPT`/`CHAT`/`WORLD` change intends it.
- **Dioxus UI reality (TEST-6).** Dioxus desktop/mobile has no mature end-to-end driver, so the
  strategy is to keep components thin and push logic into pure functions tested directly; the visual
  and interaction contract (`UI`) is verified by a per-platform manual smoke checklist at
  `docs/MANUAL_SMOKE.md`. A `VirtualDom`-level render smoke (mount a screen, assert it builds and key
  text/affordances are present) is acceptable as a supplement, not a substitute. Revisit automated UI
  testing if the Dioxus tooling matures.
- **Reference local check set:** format check, build, lint (deny warnings), and the test suite, run
  per target where feasible; mobile targets at least build in CI. Non-normative — TEST-1..17 are the
  contract.
- **What carries over from Soulfire-OG / csvz philosophy.** Same Rust-first, spec-traceable,
  deterministic, ephemeral-state approach; the external-service substitution is the **AI (and image)
  provider** rather than an identity provider, and there is no server/API/multi-tenant surface to test
  (`PROD-11`).
