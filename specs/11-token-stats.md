# Token Statistics

**Purpose:** define the new token-statistics feature: what usage is captured, how it is aggregated,
and where it is surfaced. The per-request record is owned by `DATA`; metering by `AI`; screens by
`UI`.

**Scope note:** this feature tracks **token counts only**. It does **not** compute or display any
cost or cost estimate. Cost estimation is intentionally deferred — Soulfire-OG's per-model price
tables are not reliable, and accurate cost would require maintained, model-specific pricing. Tracking
input, cached, and output tokens accurately now is the prerequisite that makes cost estimation
feasible later.

## Requirements

### Capture
- **STAT-1** Every metered AI call records a usage entry (`DATA-20a`, `AI-15`) with its timestamp,
  label (operation kind), model id, input tokens, output tokens, cached-input tokens where reported,
  and an associated chat id where applicable. This includes chat replies, summaries, character-state
  updates, adventure narration and state updates, GM commands, builder turns, and image generations.
- **STAT-2** Usage entries are retained locally and are never reset implicitly; the user may
  explicitly clear usage history, with confirmation (`UI-7`).
- **STAT-3** Token counts are captured **accurately and separately** for input, cached input (where
  the provider reports it), and output. Where the provider reports cached-input tokens as a subset of
  input tokens, the relationship is recorded unambiguously so totals are not double-counted.

### Aggregation
- **STAT-4** The app computes aggregate statistics over usage entries: totals (requests, input
  tokens, cached-input tokens, output tokens); breakdowns by **model**, by **operation label**, and
  over **time** (e.g. per day/month); and per-**chat** and per-**adventure** rollups. Aggregations are
  derived from STAT-1 entries (single source of truth) and are consistent with them. No monetary
  figure is produced.

### Surfaces
- **STAT-5** A **Token Statistics** screen (reachable from settings, `UI-20`) presents the aggregates
  in STAT-4: at minimum overall token totals (input / cached / output), a by-model breakdown, a
  by-operation breakdown, and a time trend. It supports clearing history (STAT-2). It displays no
  cost.
- **STAT-6** Contextual usage is surfaced where it is most useful: a chat and an adventure each show
  their own token rollup; the **prompt viewer** shows the estimated token size of the assembled prompt
  and per section (`PROMPT-11`). These contextual figures reconcile with the aggregate screen.

## Acceptance criteria

- **AC-STAT-a** (STAT-1, STAT-3) Performing one of each metered operation creates one usage entry per
  call with the correct label, model id, and non-zero token counts, with input, cached-input, and
  output recorded separately.
- **AC-STAT-b** (STAT-4) The statistics screen's token totals equal the sum of individual entries; the
  by-model and by-operation breakdowns partition the totals; no cost figure appears anywhere.
- **AC-STAT-c** (STAT-2) Clearing history (after confirmation) empties the aggregates; capture resumes
  on the next call.
- **AC-STAT-d** (STAT-6) A given chat's rollup equals the sum of that chat's usage entries and
  reconciles with the aggregate screen.

## Design notes (non-normative)

- Reuses Soulfire-OG's `MetricsRecord` shape and the `ChatStats`/`ChatMetrics` aggregation ideas, but
  the audience/cohort analytics (active users, acquisition, world popularity, etc.) are server
  multi-tenant concerns and are out of scope (`PROD-11`). The AI-evaluation A/B feature is also out of
  scope.
- Cost estimation is a deliberate future extension. The data captured here (per-entry input/cached/
  output tokens + model id) is exactly what a later cost layer would need; when added, cost would be a
  derived view over these same entries using a maintained pricing table, requiring no change to
  capture. Soulfire-OG's per-model price tables should **not** be ported, as they are inaccurate.
- Aggregation can be computed on demand from the entries table (indexed by time/model/label) or via a
  cached rollup; either satisfies STAT-4 as long as figures reconcile with the entries.
