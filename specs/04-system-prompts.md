# System Prompts

**Purpose:** define how prompts are assembled for chats and adventures, which sections are locked
(required for the app to function) versus editable, the content toggles (including adult content),
and the new prompt viewer/editor feature. Call sequencing and streaming are owned by `CHAT`/`WORLD`;
the provider contract is owned by `AI`.

The prompt content and section structure here are reproduced faithfully from Soulfire-OG. Section
**headers** (e.g. `## How to Be This Character`) are contract anchors; the full body text of locked
sections is carried verbatim from Soulfire-OG in implementation (it is the behavioral reference,
`PROD-7`).

## Requirements

### Prompt composition is sectioned and ordered
- **PROMPT-1** A prompt is assembled from discrete, named **sections**, concatenated in a fixed
  order. The order is durable-first so a stable prefix can be cached by the provider (`AI-4`).
  Reordering sections is not permitted, because it both changes behavior and defeats prefix caching.
- **PROMPT-2** Each section is classified as **locked** (always present, not user-editable; required
  for correct app behavior) or **editable** (user-authored or user-togglable). The classification is
  fixed per section and is part of the contract surfaced by the viewer (PROMPT-9).

### Character chat prompt
- **PROMPT-3** The character-chat prompt is composed, in order, of: (a) an optional **World Context**
  section (header `## World Context`) when the character originated from a world, wrapping that
  world's prompt — *locked*; (b) an optional **Your Character Profile** section
  (`## Your Character Profile`) wrapping the character's `extracted_context` — *locked*; (c) the
  **Character Prompt** section (`## Character Prompt`) wrapping the user's editable
  `Character.prompt` — *editable* (this is the primary authored persona); (d) the **behavior
  instructions** section (`## How to Be This Character`) — *locked* except for the toggled
  sub-behaviors in PROMPT-6; (e) the **Reactions** section (`## Reactions`) defining the optional
  trailing-emoji rule over the allowed emoji set (`DATA-6`) — *locked*; then (f) an optional
  **world-state** block (current world state + story-so-far) when the character is world-linked —
  *locked*; (g) an optional **Your Current State** section (`## Your Current State`) wrapping the
  evolving `character_state` — *locked*; (h) the recent message history; (i) the current user
  message (with any reactions). History and the current message are *not* part of the editable/locked
  classification — they are conversation data.
- **PROMPT-4** The locked **behavior instructions** reproduce Soulfire-OG's character behavior block:
  voice/presence (speak in first person as the character, not as an assistant or narrator),
  depth/engagement, the mature-roleplay stance (PROMPT-6), what-not-to-do (no third-person action
  narration, no meta-breaking, not sycophantic), and response-length guidance. The non-toggled parts
  are fixed.

### Adventure (game-master) prompts
- **PROMPT-5** Adventure prompts (narration, state-update, GM-command) reproduce Soulfire-OG's
  structure and the three reusable stance blocks: the intensity/agency-balance guidance, the
  consent-gating ban, and the mature-roleplay stance. Their composition and JSON contracts are
  detailed in `WORLD`; their locked/editable classification and the content toggles (PROMPT-6) apply
  here identically. The world blueprint prompt embedded in narration is *editable content* (it is the
  user's world); the surrounding game-master instructions are *locked*.

### Content toggles
- **PROMPT-6** The app exposes user **content toggles** in settings (`DATA-18`) that switch defined
  sub-behaviors of the locked instruction blocks on or off. At minimum an **Adult content** toggle
  controls whether the mature-roleplay stance (explicit/mature material permitted, no
  sanitizing/moralizing) is included. When a toggle is **off**, the corresponding stance text is
  omitted (or replaced by its safe counterpart) in every prompt that would include it (character
  chat and adventures). Toggle state applies to all subsequent AI calls.
- **PROMPT-7** Toggles only ever gate **clearly-delimited** sub-sections of locked blocks; they can
  never remove a section the app requires to function (e.g. the structural wrappers, the JSON-output
  contracts for state updates, the reactions rule). The set of toggle-controlled sub-sections is
  fixed and enumerable.
- **PROMPT-8** The default state of the Adult-content toggle is a single product default applied on
  first run; the user can change it at any time and the change is durable (`DATA-18`).

### Prompt viewer / editor (new feature)
- **PROMPT-9** For any character and any adventure, the user can open a **prompt view** that shows
  the fully-assembled prompt that would be sent for the next turn, broken into its named sections in
  order, each labeled **locked** or **editable**, and each labeled with its current source (authored
  field, toggle, world prompt, extracted context, dynamic state, etc.).
- **PROMPT-10** From the prompt view, **editable** sections are editable in place and saved back to
  their backing field: the Character Prompt section saves to `Character.prompt`; toggle-controlled
  sub-sections are switched via their toggles; the world blueprint content links to the world editor.
  **Locked** sections are read-only and visibly marked as required.
- **PROMPT-11** The prompt view shows an **estimated token count** for the assembled prompt and per
  section, using the selected model's tokenizer basis (`AI-16`), so the user can see how much of the
  context budget their edits and the dynamic context (history, summaries, state) consume. (Token
  counts only — no cost figure, see `STAT`.)
- **PROMPT-12** Editing an editable section through the prompt view has exactly the same effect as
  editing the corresponding field through the normal editor (`CHAR`/`WORLD`); there is one source of
  truth per field (`DATA-23`).

## Acceptance criteria

- **AC-PROMPT-a** (PROMPT-1, PROMPT-3) For a world-linked character with a dynamic state, the
  assembled prompt contains the named sections in the specified order; for a plain character, the
  optional sections are absent but the order of present sections is unchanged.
- **AC-PROMPT-b** (PROMPT-6, PROMPT-8) Turning the Adult-content toggle off removes the mature-stance
  text from both a character-chat prompt and an adventure narration prompt; turning it on restores
  it; the setting persists across restart.
- **AC-PROMPT-c** (PROMPT-7) With every toggle off, the structural wrappers, the reactions rule, and
  the adventure state-update JSON contract are still present.
- **AC-PROMPT-d** (PROMPT-9, PROMPT-10) The prompt view lists each section with a correct
  locked/editable label; editing the Character Prompt section and saving changes
  `Character.prompt`; locked sections cannot be edited.
- **AC-PROMPT-e** (PROMPT-11) The prompt view shows a non-zero total token estimate that increases
  when the user enlarges an editable section.
- **AC-PROMPT-f** (PROMPT-12) A change made in the prompt view appears in the normal editor and vice
  versa.

## Design notes (non-normative)

- Source references in Soulfire-OG: character prompt assembly in
  `services/chat/character_service.rs` (`build_character_prompts`, the `## …` section wrappers, and
  `build_character_behavior_instructions`); adventure prompts and the three stance constants in
  `services/adventure_prompts.rs`. Port these strings verbatim into a structured prompt-builder where
  each section is a value with `{name, locked, body, source}` so the viewer (PROMPT-9) can render the
  same structure the builder emits — guaranteeing the view matches what is sent.
- Implement toggles as conditional inclusion of named sub-section constants, not as string surgery on
  a monolithic prompt, so PROMPT-7 holds by construction and the viewer can show exactly which
  sub-sections a toggle controls.
- The "locked" concept maps to Soulfire-OG reality: today only `Character.prompt` (and world prompt /
  initial message) are user-editable; the behavior block, reactions rule, and section wrappers are
  server-side and effectively locked. The new feature makes that boundary visible and adds toggles.
