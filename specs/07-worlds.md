# Worlds

**Purpose:** define the worlds pillar: blueprints, adventures, the turn engine, the live
adventure-state schema, the memory ladder, the state validator, out-of-band `/gm` commands, and the
world builder. Entities are owned by `DATA`; the model layer by `AI`; prompt section classification
by `PROMPT`; covers by `IMG`; screens by `UI`.

Prompt content and JSON contracts here are reproduced faithfully from Soulfire-OG; the long
game-master instruction text is carried verbatim in implementation (`PROD-7`).

## Requirements

### Blueprints
- **WORLD-1** A **WorldBlueprint** (`DATA-8`) is an authored, reusable world template. One blueprint
  spawns many independent **Adventures** (`DATA-10`). A blueprint encodes **starting conditions, hard
  rules, act structure, and background** — never the live world.
- **WORLD-2** The user can create/edit a blueprint manually (sections **Details**, **World Prompt**,
  optional **Settings**) and from optional **templates** (Fantasy / Sci-Fi / Mystery / Horror) that
  prefill title/description/prompt. The blueprint's `description` is shown to the player and not sent
  to the AI; the `world_prompt` is the full content sent to the AI. (Soulfire-OG's
  visibility/moderation/rating controls are removed, `PROD-11`.)

### Starting an adventure
- **WORLD-3** Starting an adventure from a blueprint: the app resolves the model (`AI-9`); builds the
  player-profile context from the **PlayerProfile** (`DATA-17`) or generates a sensible default when
  blank; makes an **intro narrative** call (second-person game-master voice; strict adherence to the
  blueprint's rules; introduces the scene and the player; ends prompting the first action); and makes
  an **initial-state** call that produces the live `adventure_state` (WORLD-7) containing only what
  the player would know at the start (no unrevealed locations, NPCs, quests, or secrets).
- **WORLD-4** The new Adventure snapshots the blueprint `world_prompt`, `player_name`, and
  `player_attributes` at start (`DATA-10`), starts with empty memory layers, and persists the intro
  as the first `narration` message.

### The turn engine
- **WORLD-5** A player **turn** proceeds in this observable order: (a) the player's action is
  persisted and echoed immediately; (b) a per-adventure single-flight **lock** is claimed (a turn is
  refused while another is in progress, with a stale-lock expiry so a crashed turn self-heals); (c)
  the **narration** is generated and **streamed** (`AI-10`) and persisted as a `narration` message;
  (d) a separate **state-update** phase reconciles the live state and memory. Narration is committed
  before state update; **state-update failure is non-fatal** — the player keeps the narration and the
  state self-heals on a later turn. Turns are never replayed wholesale.
- **WORLD-6** Each turn produces exactly one `user_action` message and one `narration` message, an
  updated (or, on failure, unchanged) `adventure_state` and memory, and possibly a `story_status`
  transition. When `story_status` becomes `success` or `failure`, the adventure is marked completed
  (sticky) and an end state is shown (`UI`).

### Live adventure-state schema
- **WORLD-7** `adventure_state` is compact JSON (not required to be human-readable) with a fixed set
  of top-level sections, reproduced from Soulfire-OG: **player** details (name, attributes, traits,
  stats, skills, inventory), **quest** details (including a `current act` starting at 0, active
  quests/milestones), a structured **`current_situation`** (`location`, `time`, integer `day`
  starting at 1, `present` NPCs, `atmosphere`, `context`), **npcs** (only those the player has met,
  with relationship/attitude/personality), **story threads** (start empty; temporary side-stories;
  max 3–4 active), **`gm_notes`** (the GM's private near-term planning, max 4–6 items), and
  **flags/variables**. Inventory items are objects (e.g. `{name, type}`).
- **WORLD-8** The blueprint defines hard rules (physics, what is possible, act sequencing) that are
  enforced strictly; soft state (attitudes, relationships, locations, alliances, conditions) evolves
  and lives in `adventure_state`. The engine must never regress the live world back to the
  blueprint's initial scenario.

### Memory ladder
- **WORLD-9** The adventure maintains three distinct memory stores, reproduced from Soulfire-OG:
  - **Recent events** (`recent_summary`): newest-first short continuity lines, capped at 20.
  - **Significant events** (`significant_events`): long-term entries `{id, text, weight}` with
    stable `evt_N` ids and weight 1–5, capped at 30 via weighted age-decay pruning (priority falls
    as events age relative to `next_significant_event_id`).
  - **Story summary** (`story_summary`): a markdown blob with a `## Rolling Story` recap (a few
    past-tense third-person paragraphs) and a `## Recent Turns` newest-first list capped at ~5.
- **WORLD-10** Memory stores tolerate legacy/garbled input (a non-JSON value is coerced to a single
  entry) and are never silently wiped: a state update that returns empty memory while prior memory
  was non-empty keeps the prior memory.

### State-update reconciliation (diff-first with full fallback)
- **WORLD-11** The state-update phase chooses a path: if the live state is large (Soulfire-OG
  threshold ≈10,000 chars) a compaction directive is injected; if a fixed number of diff updates have
  accumulated (Soulfire-OG threshold 15) a **full** reconciliation runs; otherwise a **diff** update
  runs and, on any diff error, falls back to a full reconciliation. State-update calls run at low
  temperature (Soulfire-OG ≈0.15) and request JSON.
- **WORLD-12** A **diff update** returns dot-notation **patches** plus memory updates. Each patch has
  a `path` (dot-delimited; numeric segments index arrays), an op (**set** default, **append**,
  **remove**), and a value; setting a value to null removes a key; append spreads array values;
  remove drops elements matching the value (object patterns match partially, primitives match
  exactly). The response also carries `new_recent_events`, a significant-events ops object
  (`add`/`update`/`remove`), an updated `story_summary`, and a `story_status`. Patches are applied
  sequentially with **no partial commit on failure** (first failure aborts the diff and triggers the
  full fallback). The parser tolerates fenced JSON and rescues top-level memory fields that the model
  mistakenly nested inside `patches`.
- **WORLD-13** A **full update** returns the entire replacement `adventure_state` plus complete memory
  arrays and `story_status`; it is trusted as a whole-object replacement (subject to the no-wipe guard
  WORLD-10). After a full update the diff counter resets.

### State validator
- **WORLD-14** A **validator** applies diff patches to the parsed state and verifies the result before
  commit: malformed paths, out-of-range array indices, and a non-object root are rejected, causing the
  diff to abort and fall back to full reconciliation (WORLD-11). The validator is the single place
  where stronger invariants (e.g. inventory/currency conservation) can later be enforced; for launch
  it reproduces Soulfire-OG's behavior (structural validation; conservation enforced via prompt
  guidance rather than hard checks).

### Out-of-band game-master commands (`/gm`)
- **WORLD-15** Input beginning with `/gm ` is an **out-of-band game-master request** (table-level:
  skip time, retcon, fix continuity, change a rule), distinct from an in-world action. `/gm` with no
  text warns the user to add a request; an unknown `/x` command warns "unknown command". Plain text is
  an ordinary action (WORLD-5).
- **WORLD-16** A `/gm` request is first **classified** (answer-only vs. changes adventure-state vs.
  changes blueprint vs. both). An **answer-only** request returns a GM response with no state change.
  A change request returns a **proposal**: a GM response plus proposed full replacements for the
  adventure-state and/or the adventure's private blueprint copy and memory.
- **WORLD-17** A change proposal is **staged, not auto-applied**: the app computes a human-readable
  **diff** (`{target, path, before, after}` entries; structured diff for state, text diff for the
  blueprint prompt) and presents **Accept**/**Reject**. Accept applies the proposal (and may overwrite
  the **adventure's private** `world_prompt` copy only — never the source blueprint, `DATA-10`).
  Reject changes nothing. Either way a `game_master_response` message records the outcome.
- **WORLD-18** There is **no in-world undo** for adventures: ordinary actions are committed and
  irreversible. Continuity is corrected only via `/gm` retcons (and the staged-proposal mechanism).

### Save / load / resume
- **WORLD-19** Adventure state is continuously persisted server-side equivalently (here: locally) on
  every turn; there is no separate save action. The user can leave and **resume** an adventure to its
  last state with its recent turn log. A "continue where you left off" surface lists in-progress
  adventures (`UI`).

### World builder
- **WORLD-20** The user can create/refine a blueprint through a **builder** with two tabs: **Chat**
  (converse and let the assistant revise the world) and **World Prompt** (inspect the current
  blueprint). The builder is reachable as a standalone "World Builder" entry and from a blueprint's
  manual editor; the two are mutually reachable.
- **WORLD-21** Each builder turn produces a structured result with a conversational
  `assistant_message` and optional full-replacement `title`/`description`/`world_prompt` (null =
  unchanged; a change is a complete replacement, `AI-5`), bounded to the `DATA-8` limits. Before
  applying a change, the prior blueprint state is **snapshotted** (`DATA-15`); **Undo** restores the
  most recent snapshot. Caps from `DATA-15` apply.

## Acceptance criteria

- **AC-WORLD-a** (WORLD-3, WORLD-4) Starting an adventure produces an intro `narration` message and a
  non-empty `adventure_state` containing the WORLD-7 sections; the state excludes blueprint content
  the player could not yet know; the blueprint prompt is snapshotted onto the adventure.
- **AC-WORLD-b** (WORLD-5, WORLD-6) A turn echoes the action immediately, streams and persists a
  narration, then updates state; a second action submitted mid-turn is refused until the first
  completes; forcing a state-update failure preserves the narration and leaves state unchanged.
- **AC-WORLD-c** (WORLD-9, WORLD-10) Over several turns, recent events stay ≤20 newest-first,
  significant events stay ≤30 with stable ids and weights, and the story summary keeps both sections;
  a state update returning empty memory does not wipe existing memory.
- **AC-WORLD-d** (WORLD-11, WORLD-12, WORLD-14) A normal turn applies a diff; a diff with a bad path
  aborts without partial commit and a full reconciliation runs instead; after the configured number of
  diffs a full update runs and resets the counter.
- **AC-WORLD-e** (WORLD-15, WORLD-16, WORLD-17, WORLD-18) `/gm skip to morning` yields a staged
  proposal with a readable diff and Accept/Reject; Accept applies it (and can alter only the
  adventure's blueprint copy); Reject leaves everything unchanged; there is no action-undo affordance.
- **AC-WORLD-f** (WORLD-19) Leaving and reopening an adventure restores its state and recent turns;
  in-progress adventures appear in the continue surface.
- **AC-WORLD-g** (WORLD-20, WORLD-21) A world-builder turn that rewrites the prompt applies it and
  pushes a snapshot; Undo restores the prior prompt; the World Prompt tab reflects the current
  blueprint.

## Design notes (non-normative)

- Mirrors Soulfire-OG `services/adventure_service/` (`mod.rs` turn engine, `memory.rs` ladder,
  `ai_response.rs` JSON parsing/rescue), `services/adventure_prompts.rs` (narrative, full/diff
  state-update, GM classify/answer/proposal instructions, and the three stance constants),
  `services/state_patch.rs` (validator: set/append/remove, dot/index paths, `verify_state` hook),
  `services/world_builder_service.rs`, and the play/builder UIs. Constants to reuse: full-update
  threshold 15, compaction threshold ≈10,000, recent cap 20, significant cap 30 (decay 5), recent-turns
  cap ≈5, narration temp ≈0.9, state-update temp ≈0.15, stale-lock expiry ≈90s, builder temp ≈0.8.
- Per-turn AI calls: intro narrative + initial state on start; narration (streamed) + diff/full state
  update per action; classify + answer/proposal for `/gm`. Background passes are serialized per
  adventure via the ready-status lock (`AI-14`).
- Soulfire-OG multiplayer-orchestrator leftovers in the patch parser and the
  publication/moderation/rating subsystems are dropped (`PROD-11`). Prompt-prefix caching keyed per
  adventure should be preserved to reduce token usage (`AI-4`).
