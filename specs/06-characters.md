# Characters

**Purpose:** define character management: the manual editor, the conversational builder with undo,
and NPC extraction from worlds. The character entity is owned by `DATA`; chat behavior by `CHAT`;
prompt assembly by `PROMPT`; images by `IMG`; screens by `UI`.

## Requirements

### Manual editor
- **CHAR-1** The user can create and edit a character through a manual editor organized into the
  sections **Profile**, **Prompt**, **Initial Message**, and **Settings**, matching Soulfire-OG.
- **CHAR-2** **Profile** edits: `name`, `subtitle` (noted as not used by the AI), `description`
  (noted as not used by the AI), and avatar selection — an emoji avatar (`DATA-20`) or a generated
  portrait with crop/transform (`IMG`).
- **CHAR-3** **Prompt** edits the character's `prompt` (`DATA-1`) — the core personality
  instructions. **Initial Message** edits the `initial_message` (`DATA-2`) with a Direct/Prompt type
  toggle and the opening content.
- **CHAR-4** **Settings** exposes the creativity controls — max output tokens, temperature, top-p
  (and top-k is accepted/clamped per `DATA-1` even if not surfaced as a field) — within their valid
  ranges. Soulfire-OG's visibility/public controls are removed (`PROD-11`).
- **CHAR-5** Saving validates required fields (name and prompt and initial message non-empty) and
  applies the `DATA-1` clamps; invalid input is rejected with a clear message and nothing is saved.

### Conversational builder
- **CHAR-6** The user can create or refine a character through a **builder**: a chat where the user
  describes what they want and the assistant replies conversationally and/or revises the character.
  The builder is reachable both as a standalone "Character Builder" entry and from an existing
  character's editor; the editor and builder are mutually reachable.
- **CHAR-7** Each builder turn produces a structured result containing a conversational
  `assistant_message` plus optional full-replacement values for `name`, `subtitle`, `description`,
  `prompt`, and `initial_message`; a null field means "leave unchanged", and a changed `prompt` or
  `initial_message` is a complete replacement, not a patch (`AI-5`). Applied changes are validated and
  clamped (CHAR-5).
- **CHAR-8** Before applying changes, the builder **captures a snapshot** of the prior character
  state onto the session's snapshot stack (`DATA-14`); an **Undo** action restores the most recent
  snapshot and notes the restoration in the conversation. Undo is disabled when no snapshots exist.
  The message log and snapshot stack honor their caps (`DATA-14`).
- **CHAR-9** The builder uses structured-JSON generation (`AI-5`) and bounds the per-field lengths to
  the `DATA-1`/`DATA-2` limits (the assistant is instructed to respect them, and the app enforces
  them on apply).

### NPC extraction from worlds
- **CHAR-10** From within an adventure (once it has progressed), the user can **extract an NPC** by
  name into a standalone chat character (the "Bring a Character to Life" flow). Extraction reads the
  adventure and its blueprint and produces, via model calls: an immutable **persona profile**
  (stored as `extracted_context`) capturing identity, voice, emotional patterns, key memories, and
  motivations; and an initial **dynamic state** (stored as `character_state`) capturing current
  emotional state, relationship to the player, concerns, and unresolved threads.
- **CHAR-11** The extracted character records its origin (`DATA-4`: `source_blueprint_id`,
  `source_adventure_id`, `source_npc_name`), is given a `prompt` and a Prompt-type `initial_message`
  consistent with that origin, and a chat is created and opened with a generated opening message and
  title. The character then behaves like any other character, including the background character-state
  updater (`CHAT-12`).
- **CHAR-12** Extraction runs asynchronously and notifies the user when the character is ready; a
  failed extraction surfaces an error and creates no partial character.

### Browsing
- **CHAR-13** Characters are listed with avatar, name, origin/subtitle, description, and last-chat
  time, with search and incremental loading. Row actions include opening/continuing the chat, editing
  the character, regenerating its portrait (`IMG`), deleting its chat, and deleting the character
  (destructive actions confirm first). (Screen details in `UI`.)

## Acceptance criteria

- **AC-CHAR-a** (CHAR-1..5) A character created via the manual editor saves all four sections;
  out-of-range creativity values are clamped; empty name/prompt/initial-message is rejected.
- **AC-CHAR-b** (CHAR-7, CHAR-8) A builder turn that changes the prompt applies it and pushes a
  snapshot; Undo restores the prior prompt; Undo is disabled with no snapshots; exceeding the caps
  drops the oldest entries.
- **AC-CHAR-c** (CHAR-10, CHAR-11) Extracting a named NPC produces a character with non-empty
  `extracted_context` and `character_state`, recorded origin fields, and an opened chat with an
  opening message and title.
- **AC-CHAR-d** (CHAR-12) A forced extraction failure produces an error and leaves no new character or
  chat.
- **AC-CHAR-e** (CHAR-13) The character list searches, paginates, and offers the listed row actions
  with confirmation on deletes.

## Design notes (non-normative)

- Mirrors Soulfire-OG `services/chat/character_builder_service.rs`,
  `db/chat/character_builder_session.rs`, `services/character_extraction.rs`, and the editor/builder
  UIs under `pages/characters/`. Builder call: structured JSON, temperature ≈0.8, generous max
  tokens; extraction profile call uses the chat model at ≈0.7, the initial-state call uses the cheap
  state/utility model. Description in the builder is bounded shorter (Soulfire-OG: ≤240 chars) than
  the stored `description` max.
- The locked/editable boundary the builder exposes is the same one the prompt viewer formalizes
  (`PROMPT`): the builder writes the editable `prompt`/`initial_message`, never the locked behavior
  block.
