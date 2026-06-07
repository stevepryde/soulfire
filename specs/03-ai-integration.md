# AI Integration

**Purpose:** define the provider abstraction, OpenAI (BYOK) behavior, streaming, structured output,
the model registry, creativity controls, and request metering. Prompt *content* is owned by `PROMPT`;
feature-specific call sequences are owned by `CHAT`/`WORLD`/`CHAR`/`IMG`; persisted metrics are owned
by `DATA`/`STAT`.

## Requirements

### Provider abstraction
- **AI-1** The app talks to AI providers through a single internal contract that exposes: a
  one-shot text generation, a streamed text generation, and (where the provider supports it)
  structured-JSON generation and image generation. Every text/structured call accepts: a list of
  role-tagged messages (roles **developer/system**, **user**, **model/assistant**), an optional
  separate cacheable **instructions** block, a model selection, generation config (AI-6), and a
  metering **label** (`DATA-20a`).
- **AI-2** At launch the only registered provider is **OpenAI** (`PROD-15`). Adding a provider must
  not require changes to `CHAT`/`WORLD`/`CHAR` specs; only the registry (AI-7) and a provider
  adapter change.
- **AI-3** Every AI call uses the user's stored API key for the relevant provider (`SEC-9`). If no
  key is configured for the required provider, the call fails with a clear, user-actionable
  "add your API key" condition rather than a generic error, and no request is sent.

### OpenAI behavior
- **AI-4** OpenAI text generation maps the role-tagged messages and the separate **instructions**
  block to the provider's request such that the instructions block is the stable, cache-eligible
  prefix. The instructions block is used for prompt-prefix caching where the provider supports it,
  and prompt ordering (durable → volatile) is preserved end-to-end (the ordering contracts live in
  `PROMPT`/`WORLD`).
- **AI-5** **Structured output:** when a caller requests JSON with a schema, the request constrains
  the model to that schema (strict), and object schemas disallow unspecified properties. When a
  caller requests JSON without a schema, the response is requested as JSON text. Callers additionally
  tolerate provider output wrapped in code fences (the JSON-parsing helpers strip ```` ```json ````/
  ```` ``` ```` fences and surrounding whitespace before parsing).
- **AI-6** **Generation config** carries: max output tokens, temperature, top-p, top-k, an optional
  reasoning-effort level, optional JSON mode + schema, and optional prompt-cache hints. Parameters a
  given provider does not support are ignored without error (e.g. OpenAI ignores top-k). Defaults
  match Soulfire-OG per call site (the per-call temperatures/limits are specified in
  `CHAT`/`WORLD`/`CHAR`/`IMG`).

### Model registry
- **AI-7** The app ships a **curated registry** of supported OpenAI models (`Model Choice = curated
  list only`, per product decision). Each registry entry carries: a stable id, a human display name,
  and the vendor. There is **no plan gating**; every registered model is selectable. (No per-model
  pricing is stored — cost estimation is out of scope, see `STAT`.)
- **AI-8** The registry defines task **defaults**, matching Soulfire-OG's split: a default
  **chat/narrative** model (used for chat replies and adventure narration) and a default
  **state/utility** model (used for adventure state updates, summaries, classification, and other
  background passes — a cheaper/faster model). These defaults are used when no explicit model is
  chosen.
- **AI-9** Model selection precedence for a given operation is: the entity's stored model (e.g.
  `Chat.ai_model`, `Adventure.ai_model`) if set, else the app profile's `default_ai_model`
  (`DATA-16`) if set, else the registry task default (AI-8). The chosen model is persisted on the
  entity when an entity-scoped operation starts.

### Streaming
- **AI-10** Streamed generation delivers incremental text deltas to the caller as they arrive, a
  terminal full-text event, a terminal usage/metadata event, and error events. Callers render deltas
  live (`CHAT`/`WORLD`).
- **AI-11** Streaming enforces an **idle timeout**: if no first token arrives within the timeout, the
  operation fails with an error surfaced to the user; if the stream goes idle after partial text, the
  partial text is finalized rather than discarded. (Timeout values per surface are in `CHAT`/`WORLD`.)

### Errors, limits, and resilience
- **AI-12** The app does **not** impose its own usage caps (`PROD-13`). Provider errors — including
  authentication failures, rate-limit/quota errors, content/safety blocks, and transient
  unavailability — are surfaced to the user with a clear message and leave local state consistent
  (a failed turn does not corrupt a chat or adventure).
- **AI-13** Transient provider unavailability (e.g. HTTP 503) is retried a bounded number of times
  with backoff before surfacing an error; non-transient errors fail fast.
- **AI-14** Concurrency: long-running background passes (summaries, state updates, image generation)
  run without blocking the user's next interaction, and per-entity work is serialized so two updates
  to the same chat/adventure/character cannot interleave and corrupt state (the per-entity locks are
  specified in `CHAT`/`WORLD`).

### Metering
- **AI-15** Every metered call records a `UsageMetric` (`DATA-20a`) with its label, the model used,
  and input/output (and cached-input where reported) token counts, on both one-shot and streamed
  completion. Untracked/internal calls (if any) are explicitly excluded. This feeds token statistics
  (`STAT`).
- **AI-16** Token counting for display/estimation (e.g. previewing a prompt's size in the prompt
  editor, `PROMPT`) uses the same tokenizer basis as the selected model's provider so estimated token
  counts match the provider's reported token usage closely.

## Acceptance criteria

- **AC-AI-a** (AI-3) With no OpenAI key set, any AI action reports an actionable "add your API key"
  state and sends no network request; after a valid key is added, the same action succeeds.
- **AC-AI-b** (AI-5) A structured call returns an object conforming to the requested schema;
  a fenced ```` ```json ```` response is parsed successfully.
- **AC-AI-c** (AI-7, AI-9) Selecting a model for a chat persists it; reopening the chat uses that
  model; clearing it falls back to the profile default, then the registry default.
- **AC-AI-d** (AI-10, AI-11) A streamed reply renders token-by-token; a stream that yields no first
  token within the idle timeout surfaces an error; a stream that stalls mid-reply keeps the partial
  text.
- **AC-AI-e** (AI-12, AI-13) A simulated 429/quota error surfaces a clear message and leaves the chat
  and adventure unchanged; a simulated 503 retries then succeeds or fails cleanly.
- **AC-AI-f** (AI-15) Each user-visible AI action produces exactly the expected UsageMetric rows with
  correct labels and model ids.

## Design notes (non-normative)

- This consolidates Soulfire-OG's `services/ai` (`AiServiceTrait`, OpenAI Responses-API adapter,
  streaming event mapping, retry/backoff, structured-output schema injection, fence-stripping and
  JSON-rescue helpers) into the local app. The Gemini adapter and the global/per-user rate limiters
  and live A/B "eval" harness from Soulfire-OG are out of scope for launch (the eval harness is admin
  tooling; rate limiters are server multi-tenant concerns — `PROD-13`).
- The OpenAI adapter is a **vendored, trimmed copy** of the user's `ai-client` crate (OpenAI path
  only; the unmaintained Gemini code is dropped), wired behind the provider seam (`AI-1`). See the
  vendoring decision in [`12-platform-packaging.md`](12-platform-packaging.md) Design notes.
- Curated registry seed (validate exact ids against the OpenAI API at implementation time): a current
  flagship chat model as the chat/narrative default and a current small/cheap model as the
  state/utility default, plus a couple of mid-tier options. Because the list is curated-only, adding
  a model is a one-line registry change shipped in an app update.
- Provider safety/content posture: OpenAI applies its own provider-side policies; the app does not
  add its own content gating beyond what the user configures via the content toggles (`PROMPT`).
  Soulfire-OG's Gemini-specific safety-setting relaxations do not apply to the OpenAI-only launch.
- Keep the provider adapter free of feature logic so a future provider (Anthropic, Google, local
  models) is purely additive.
