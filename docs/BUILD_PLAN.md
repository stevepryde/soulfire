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

### Layer 3 — AI provider seam (`soulfire-core::ai`, AI) — DONE (15 ai tests)
- [x] AiProvider trait (one-shot text, streamed text, structured JSON, image) + AiService (key
  guard AI-3, transient retry AI-13) + ApiKeySource seam
- [x] OpenAI Responses adapter over ai-client; instructions/cacheable prefix; gen config; reasoning;
  error mapping (build-verified; not network-tested per TEST-4)
- [x] streaming events + idle timeout (collect_streamed); retry/backoff; fence-strip + JSON-rescue
- [x] model registry + selection precedence (AI-9); token estimate (AI-16)
- [x] recording fake provider (scripted text/stream/error/image/stall, records requests)
- Metering (UsageMetric write) is done by the calling engine (has entity context + store), per AI-15.

### Layer 4 — prompt assembly (`soulfire-core::prompt`, PROMPT) — character done (6 tests)
- [x] sectioned builder w/ {header, locked, body, source} + AssembledPrompt.instructions()/outline()
- [x] character-chat prompt in PROMPT-3 order; verbatim OG behavior/reactions/wrapper text (PROD-7)
- [x] adult-content toggle gates the mature stance by construction (PROMPT-6/7); structural sections
  always present; only Character Prompt editable
- [x] token estimation lives in ai::registry (AI-16); golden structural tests (TEST-10 character part)
- [ ] adventure (game-master) prompts + 3 stance blocks + JSON contracts — built WITH Layer 6 (WORLD)
  since they are coupled to the turn engine's diff/full/GM JSON contracts

### Layer 5 — chat engine (`soulfire-core::chat`, CHAT) — DONE (tests TEST-11)
- [x] pure helpers: sanitise_reply (trailing-emoji + list normalization, CHAT-8), history builder
  with reactions (CHAT-5/9), per-character Coalescer state machine (CHAT-13)
- [x] verbatim OG summary + character-state-update prompts (PROD-7)
- [x] ChatEngine: open_chat (verbatim vs generated opening, CHAT-1/2), send_message (persist→stream→
  finalize, reaction extraction, metering, last_chatted/updated, auto-title CHAT-3, idle timeout
  CHAT-6, output floor 2000 CHAT-7), rolling summary (CHAT-10/11), character-state updater
  (CHAT-12) with coalesced queue (CHAT-13)
- [x] 6 integration tests (opening, stream+finalize+meter, reaction→AI reaction, summary regen +
  failure-preserves, state update + failure-preserves) + unit tests
### Layer 6 — world turn engine (`soulfire-core::world`, WORLD) — DONE (tests TEST-13)
- [x] state_patch validator (WORLD-12/14), memory ladder (WORLD-9/10), tolerant response parser
  (WORLD-12/13/16), verbatim adventure prompts incl. 3 stances + JSON contracts (PROD-7)
- [x] pure input parser (WORLD-15: /gm vs unknown vs action)
- [x] WorldEngine: start_adventure (intro + initial-state, WORLD-3/4), take_turn (single-flight lock
  + stale-heal WORLD-5, echo→stream narration→non-fatal state update, diff-first w/ full fallback +
  compaction WORLD-11, memory no-wipe WORLD-10, story_status/has_completed WORLD-6), /gm classify→
  answer/proposal staged with readable diff + accept/reject (WORLD-16/17), metering
- [x] 6 integration tests (start, turn+diff, state-update-failure-non-fatal, lock refuse+self-heal,
  warnings, /gm stage/accept/reject) + ~24 unit tests
- Fixed message ordering to use SQLite rowid (insertion order) so same-timestamp messages stay
  chronological — also fixes chat.
### Layer 7 — character builder + extraction (`soulfire-core::character`, CHAR) — DONE (TEST-12)
- [x] verbatim builder + extraction prompts (PROD-7); BuilderResult structured parse
- [x] CharacterEngine: builder_send (full-replacement apply + clamp, snapshot-before-apply CHAR-8),
  builder_undo (restore snapshot, disabled when none); extract_npc (persona + initial-state, origin
  fields, create+open chat with title; failure leaves nothing partial CHAR-12)
- [x] 4 integration tests

### Layer 8 — images (`soulfire-core::image`, IMG) — DONE (TEST-14)
- [x] ImageEngine: generate/regenerate portrait+cover (version bump IMG-3, failure keeps prior IMG-2),
  local-upload set-bytes (IMG-6), clear-to-emoji (IMG-8), metering; prompt builders; bytes encrypted
  at rest (proven on disk). 4 tests.
### Layer 9 — stats aggregation (`soulfire-core::stats`, STAT) — DONE (2 tests)
- [x] totals/by-model/by-label/by-day(+month) + StatsReport; cached subset of input (STAT-3); no cost
### Layer 10 — starter worlds seed (`soulfire-core::seed`, ONB) — DONE (3 tests)
- [x] Beneath Verath lead starter (authored, OG act format); seed_starter_worlds idempotent, ledger-
  tracked, no-duplicate, no-resurrect-deleted (ONB-5/DATA-21). CORE COMPLETE: 172 tests.

### Layer 11 — Dioxus UI (`app`, UI + ONB surfaces) — IN PROGRESS (compiles clean)
- [x] theme tokens + accent system (ported OG input.css + compiled CSS); app shell; nav
  (sidebar/bottom); title bar; immersive vs standard surfaces; toast container
- [x] lock + first-run setup (password + key + seed); engine wiring (AppContext + store-backed keys)
- [x] worlds home (tabs/cards/start/continue); immersive play (streamed narration, message-by-type,
  /gm proposal accept/reject, composer status, NPC extraction); characters list; immersive chat
  (streamed, markdown bubbles, emoji reactions); character + world editors; settings (accent, masked
  key, model, adult toggle, editable adventure defaults); profile + lock; token stats
- [x] prompt viewer/editor (PROMPT-9/10/11); confirmation dialogs (UI-7); docs/MANUAL_SMOKE.md (TEST-6)
- [x] first-run story name-capture + auto-start (ONB-2/3/4); conversational character + world
  builders with Chat/Inspect + undo (CHAR-6/WORLD-20, incl. WorldBuilderEngine in core); NPC
  extraction from play; image generate/regenerate/clear + pan/zoom/reset framing (IMG-1..3/7/8);
  composer draft persistence (DATA-26); list search (UI-8/17); render smoke test (TEST-6)
- [ ] remaining polish: card thumbnails for stored portraits/covers; drag-to-pan (sliders for now);
  adventure prompt viewer; "load more" pagination; bundle Inter/Merriweather fonts

### Layer 12 — packaging (PKG) — STARTED
- [x] Dioxus.toml (app/bundle ids, tailwind input/output); compiled CSS committed
- [ ] per-target build/bundle config (5 targets); font bundling; data-location doc; forward-migration
  test; mobile build verification in CI

## Notes / decisions
- OG `AiModel` carried pricing + Gemini; rebuild drops both (OpenAI-only, no cost — STAT). Registry
  holds id/display/vendor only; map AiModel → ai-client OpenAI request model in the adapter.
- OG models are user-keyed; rebuild removes `user_id`, `visibility`, plan/feature flags; profiles +
  settings + install-state are singleton rows.
