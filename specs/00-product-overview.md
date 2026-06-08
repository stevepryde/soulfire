# Product Overview

**Purpose:** define what Soulfire (local) is, the platforms it targets, what is in and out of scope
relative to Soulfire-OG, and the licensing posture.

## Requirements

### Identity & shape
- **PROD-1** The product is named **Soulfire**. It is a single-user, locally-installed application;
  there are no user accounts, no sign-in, no server-side component operated by the project, and no
  network dependency other than the user's chosen AI provider.
- **PROD-2** Soulfire is **BYOK** (bring-your-own-key): the user supplies their own AI provider API
  key(s). The app never ships or proxies keys, and makes AI calls directly from the local device to
  the provider.
- **PROD-3** Soulfire runs on **Windows, macOS, Linux, Android, and iOS** as a Tauri v2 application:
  Rust owns the durable product core and local storage, and a React/TypeScript interface owns
  presentation and interaction (see `PKG`/`UI`).
- **PROD-4** All persistent state lives in a single local encrypted database on the device (see
  `SEC`, `DATA`). No data leaves the device except the content the user sends to their AI provider.

### Feature pillars (faithful to Soulfire-OG)
- **PROD-5** Soulfire reproduces Soulfire-OG's two experience pillars: **Character Chat** (1:1
  persona conversation, see `CHAT`/`CHAR`) and **Worlds** (persistent interactive-fiction adventures
  with a turn engine, see `WORLD`), including the conversational **builders** for both, and **NPC
  extraction** from worlds into chat characters.
- **PROD-6** Soulfire reproduces Soulfire-OG's look, feel, and screen flows faithfully: the same dark
  theme with a user-selectable accent color, the same immersive vs standard surface idioms, the same
  navigation model, and the same screens, adapted to desktop and mobile (see `UI`).
- **PROD-7** Soulfire reproduces Soulfire-OG's AI behavior faithfully: the same prompt assembly,
  streaming, structured-output handling, memory/summary cadences, and creativity controls (see `AI`,
  `PROMPT`, `CHAT`, `WORLD`).
- **PROD-8** Soulfire reproduces Soulfire-OG's AI media generation: AI-generated character portraits
  and world cover images, with the same crop/transform editors, stored locally (see `IMG`).

### New local-only features
- **PROD-9** Soulfire adds **token statistics**: per-request and aggregate token usage are captured
  and viewable in the app (see `STAT`). Costs are NOT estimated.
- **PROD-10** Soulfire adds a **system-prompt viewer/editor**: users can inspect the full prompt that
  will be sent for a given chat or adventure. Sections required for the app to function are shown
  read-only ("locked"); other sections are editable, and selected behaviors (including **adult
  content**) are exposed as user toggles (see `PROMPT`).

### Scope boundaries (dropped from Soulfire-OG)
- **PROD-11** The following Soulfire-OG subsystems are **out of scope** and must not appear in the
  product: user accounts / OIDC / MFA / sessions, billing / subscriptions / plans / usage caps,
  admin tooling, content moderation and review queues, world ratings, and public/shared worlds or
  characters. Any data field, screen, route, or behavior whose sole purpose is one of these is
  removed.
- **PROD-12** Because the app is single-user, every Soulfire-OG concept keyed per user collapses to a
  singleton: there is exactly one app profile and exactly one worlds (player) profile. Per-user
  uniqueness constraints become global constraints. "Visibility" (private/public) is removed; all
  content is local and private by definition.
- **PROD-13** Soulfire-OG's daily/global rate limits are removed. The only limits that apply are
  those imposed by the user's own AI provider account; the app surfaces provider errors (including
  rate-limit and quota errors) to the user rather than enforcing its own caps (see `AI`).
- **PROD-14** The Soulfire-OG "chat modes" named in product docs (Lesson, Corrections, freeform
  Conversation, in-chat Storyteller) that have **no implementation** in Soulfire-OG are **not**
  built. The shipped conversational surfaces are exactly Character Chat and the Worlds adventure
  engine, matching Soulfire-OG's actual behavior.

### Provider scope
- **PROD-15** At launch, the only supported AI provider is **OpenAI**. The provider layer is designed
  so additional providers can be added later without changing feature specs (see `AI`).

### Licensing
- **PROD-16** Soulfire is open source: the source is publicly available and licensed under **"MIT OR
  Apache-2.0"** (dual license, user's choice). The repository ships both license texts and declares
  the dual license in its package metadata and README.
- **PROD-17** The user interface never owns durable product truth directly: it may hold view state,
  form drafts, optimistic affordances, and request status, but persisted entities, prompt assembly,
  AI request construction, credentials, and storage are owned by the Rust core. The interface must
  not access the database, persist API keys, or assemble production prompts itself.

## Acceptance criteria

- **AC-PROD-a** (PROD-1, PROD-4, PROD-11) A fresh install with no network access to anything except
  the configured AI provider is fully usable for all non-AI features (browsing, editing, reading
  saved content); no screen, route, or setting references accounts, billing, admin, moderation,
  ratings, or public content.
- **AC-PROD-b** (PROD-2, PROD-15) With a valid OpenAI key entered, chat and adventures function;
  with no key entered, the app clearly prompts the user to add a key before any AI action.
- **AC-PROD-c** (PROD-3) The same codebase builds and launches on all five target platforms (see
  `PKG` acceptance).
- **AC-PROD-d** (PROD-12) The app exposes exactly one app profile and one player profile; there is no
  affordance to create a second of either.
- **AC-PROD-e** (PROD-16) The repository root contains both `LICENSE-MIT` and `LICENSE-APACHE` (or
  equivalent), and package metadata declares `MIT OR Apache-2.0`.

## Design notes (non-normative)

- Soulfire-OG is a Rust workspace (`lib-soulfire` shared models, `soulfire-api` Axum backend,
  `soulfire-ui` Dioxus CSR + Tailwind). The rebuild collapses backend orchestration into the local
  app: prompt assembly, the turn engine, builders, summarizers, and the state validator move from
  the Axum services into Rust core modules exposed to the React shell through typed native commands
  and event streams. The shared model crate ports nearly intact (minus account/billing/moderation
  types).
- The reference manifest positions Soulfire as "an AI companion for meaningful conversation and
  immersive storyplay" with a north star of "self-discovery and personal growth through deep
  conversation and immersive roleplay." Keep this voice in onboarding and marketing surfaces (`ONB`,
  `UI`).
- Soulfire-OG also had a never-finished optional "Shopping List" utility module; it is intentionally
  not part of this rebuild.
- Soulfire-OG used browser APIs for things like websockets and interaction with the backend. The
  rebuild replaces those transports with Tauri command and event boundaries; React renders state and
  interaction while Rust remains the source of durable truth.
