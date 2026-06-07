# Soulfire build plan

Living roadmap for building the full app from `specs/`. Bottom-up: a testable pure-Rust core,
then the Dioxus UI shell. Soulfire-OG (`~/projects/app-world/soulfire`) is the behavioral
reference; the `specs/` are the contract. This file tracks progress so work resumes cleanly.

## Architecture

Workspace crates:

- `libs/lib-soulfire` — **domain models** (DATA): entities, typed IDs, bounded strings, enums,
  validation/clamping. Pure data + serde. Ported to the DATA-spec shapes (single-user singletons,
  no `user_id`, no visibility/billing/moderation) using OG field names. Deps: sp-core, serde,
  serde_json, strum, bon, indexmap, anyhow.
- `libs/ai-client`, `libs/sp-core`, `libs/sp-markdown`, `libs/sp-ui` — vendored (done).
- `libs/soulfire-core` — **the engine** (pure Rust, no Dioxus; the TEST-2/TEST-5 testable core):
  - `clock`, `keychain`, `provider` seams (traits) — substitutable for tests (TEST-5).
  - `store` — SQLCipher via rusqlite: crypto/KDF (Argon2id), unlock/rekey, schema+migrations,
    per-entity repositories, fresh-store init, draft/metric/credential storage (SEC, DATA).
  - `ai` — provider seam impl: OpenAI Responses adapter over ai-client, streaming, structured
    output, retry/backoff, fence-strip/JSON-rescue, metering → UsageMetric (AI).
  - `prompt` — sectioned prompt builder (character + adventure), content toggles, token est (PROMPT).
  - `chat` — chat engine: opening, send/stream, reactions, summary, char-state updater (CHAT).
  - `world` — turn engine, adventure-state schema, memory ladder, validator, diff/full reconcile,
    `/gm` flow, world builder (WORLD).
  - `character` — editor validation, builder, NPC extraction (CHAR).
  - `image` — generation, encrypted storage, transform (IMG).
  - `stats` — usage aggregation (STAT).
  - `seed` — bundled starter worlds + first-run seeding (ONB, DATA-21).
- `app` — **Dioxus desktop/mobile UI**: theme/tokens (port OG input.css), app shell + nav, all
  screens, onboarding, prompt viewer, settings, stats. Wires core into UI state. Extracted view
  logic is pure + unit-tested (TEST-6). Deps: soulfire-core, lib-soulfire, sp-ui, sp-markdown,
  sp-core, dioxus (desktop/mobile), lucide-dioxus, manganis.

Seams are traits in `soulfire-core`; production wires real OpenAI/OS-keychain/system-clock, tests
wire deterministic fakes (recording provider, fake clock, in-memory keychain).

## Progress

Legend: [ ] todo · [~] in progress · [x] done

### Layer 1 — domain models (`lib-soulfire`, DATA) — DONE (44 tests)
- [x] string_type! + bounded string newtypes; typed IDs (sp-core SpId) with DATA prefixes
- [x] enums: AiModel registry (OpenAI-only) + vendor + task defaults; CharacterImage avatars
  (20 emoji + 8 illustrated); WorldImage covers (78); Language; InitialMessage; sender/message
  types; StoryStatus; AdventureMessageType; ready status; GM change target; content-toggle keys
- [x] entities: Character, Chat, ChatMessage (Reactions + ALLOWED_EMOJIS filtering), WorldBlueprint,
  Adventure, AdventureMessage, GmProposal, CharacterBuilderSession, WorldBuilderSession, AppProfile,
  PlayerProfile, AppSettings, ProviderCredential, InstallState, UsageMetric, Draft
- [x] validation/clamping (creativity controls, length bounds, reaction filtering)
- [x] unit tests (TEST-7 pure parts): clamps, reaction filter, id prefixes, serde round-trip
- Fixed `SpDateTime::now()` to millisecond resolution so values survive serialize→parse (matches
  the persisted contract form).

### Layer 2 — core seams + store (`soulfire-core`, SEC + DATA persistence) — DONE (23 tests)
- [x] Clock (System/Mock), Keychain (trait + in-memory fake) seams; Provider trait deferred to L3
- [x] crypto: Argon2id KDF (64MiB/3/1), SHA-256 verifier, plaintext sidecar, SQLCipher raw key,
  versioned params
- [x] store: schema (one table/entity + JSON blob), migrations (user_version), repositories per
  entity, fresh-store init (DATA-25), cascade deletes (DATA-22), singletons, drafts, metrics, images
- [x] unlock/lock/rekey (PRAGMA rekey); keychain remember/forget (SEC-7); key-never-leaks
- [x] tests: TEST-7 (round-trip, cascades, singletons, draft lifecycle), TEST-8 (ciphertext-on-disk,
  wrong-password lock, rekey, device-remember)

### Layer 3 — AI provider seam (`soulfire-core::ai`, AI)
- [ ] AiProvider trait (one-shot text, streamed text, structured JSON, image)
- [ ] OpenAI Responses adapter over ai-client; instructions/cacheable prefix; gen config; reasoning
- [ ] streaming events + idle timeout; retry/backoff; fence-strip + JSON-rescue
- [ ] model registry + selection precedence (AI-9); metering → UsageMetric
- [ ] recording fake provider; tests TEST-9

### Layer 4 — prompt assembly (`soulfire-core::prompt`, PROMPT)
- [ ] sectioned builder w/ {name, locked, body, source}; character + adventure prompts (verbatim OG)
- [ ] three stance blocks; content toggles (adult default off); token estimation
- [ ] golden snapshot tests; tests TEST-10

### Layer 5 — chat engine (`soulfire-core::chat`, CHAT) — tests TEST-11
### Layer 6 — world turn engine (`soulfire-core::world`, WORLD) — tests TEST-13
### Layer 7 — character builder + extraction (`soulfire-core::character`, CHAR) — tests TEST-12
### Layer 8 — images (`soulfire-core::image`, IMG) — tests TEST-14
### Layer 9 — stats aggregation (`soulfire-core::stats`, STAT) — tests TEST-15
### Layer 10 — starter worlds seed (`soulfire-core::seed`, ONB) — tests TEST-16

### Layer 11 — Dioxus UI (`app`, UI + ONB surfaces)
- [ ] theme tokens + accent system (port OG input.css); app shell; nav (sidebar/bottom)
- [ ] lock/onboarding/first-run story; worlds home; play screen; chat screen; characters list;
  editors + builders; settings; profile; prompt viewer; token stats; image crop editors
- [ ] extracted view-logic unit tests (TEST-6); app smoke (TEST-17); docs/MANUAL_SMOKE.md

### Layer 12 — packaging (PKG)
- [ ] Dioxus.toml, assets pipeline, per-target build config; data-location; forward-migration test

## Notes / decisions
- OG `AiModel` carried pricing + Gemini; rebuild drops both (OpenAI-only, no cost — STAT). Registry
  holds id/display/vendor only; map AiModel → ai-client OpenAI request model in the adapter.
- OG models are user-keyed; rebuild removes `user_id`, `visibility`, plan/feature flags; profiles +
  settings + install-state are singleton rows.
