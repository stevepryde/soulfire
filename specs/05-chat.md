# Character Chat

**Purpose:** define the behavior of 1:1 character chat: starting a chat, sending messages, streaming
replies, reactions, the rolling summary, and the background character-state updater. The model layer
is owned by `AI`; prompt assembly is owned by `PROMPT`; the character entity and editors are owned by
`CHAR`; the chat screen is owned by `UI`.

## Requirements

### Starting and identity
- **CHAT-1** A chat is a 1:1 conversation between the **player** and exactly one **character**. At
  most one chat exists per character (`DATA-5`); starting a chat for a character that already has one
  opens the existing chat.
- **CHAT-2** When a chat is first opened, the character delivers an **opening message** derived from
  its `initial_message` (`DATA-2`): a **Message** initial is sent verbatim (no model call); a
  **Prompt** initial triggers a model call that produces the opening line using the assembled
  prompt (`PROMPT`). The opening message is persisted as the first character message.
- **CHAT-3** A chat's **title** is generated automatically: when the title is empty, the first
  exchange produces a short (≤5 word) summary title via a model call (label `chat_summary`). Until a
  title exists, the chat displays a sensible default (e.g. derived from its start time).

### Sending and streaming
- **CHAT-4** Sending a player message: the message is persisted and shown immediately; the assembled
  prompt (`PROMPT-3`) is sent with the chat's resolved model (`AI-9`) and the character's creativity
  controls (`DATA-1`); the reply is **streamed** (`AI-10`) and rendered token-by-token; on completion
  the full character message is persisted. The character's `last_chatted_at` and the chat's
  `updated_at` advance.
- **CHAT-5** Prompt history sent to the model is bounded to the most recent messages (Soulfire-OG:
  last 20), in chronological order, each tagged by sender role, with any emoji reactions appended to
  the message text.
- **CHAT-6** A streamed reply honors the idle-timeout rule (`AI-11`): no first token within the
  timeout surfaces an error and no partial message is saved; a stall after partial text finalizes and
  saves the partial reply.
- **CHAT-7** Generation max-output respects the character's `max_tokens` but is floored to a minimum
  (Soulfire-OG floor: 2000) so replies are not truncated by an unreasonably low setting.

### Reply post-processing
- **CHAT-8** A character reply may end with a single trailing **reaction emoji** from the allowed set
  (`DATA-6`); when present it is removed from the visible message text and instead recorded as the
  character's (`AI`) reaction to the player's preceding message. List-style markers in replies are
  normalized for display (Soulfire-OG converts `a) b)`-style enumerations to line breaks).

### Player reactions
- **CHAT-9** The player can react to any message with one emoji from the allowed set (`DATA-6`); the
  reaction is persisted on that message and shown under it. Reactions are included in the prompt
  history (CHAT-5) so the character is aware of them.

### Rolling conversation summary
- **CHAT-10** The chat maintains a rolling **conversation summary** (`Chat.chat_summary`) used as
  long-term memory in the prompt (`PROMPT-3` chat-context section). After every fixed interval of
  messages (Soulfire-OG: 20), a background pass regenerates the summary from the recent window
  (Soulfire-OG: last 40 messages), folding the previous summary in, and resets the
  `messages_since_summary` counter. The summary captures key topics, decisions, emotional dynamics,
  and continuing context, written in third person.
- **CHAT-11** Summary generation is a background pass (`AI-14`): it does not block the player's next
  message, and a failed summary pass leaves the prior summary and the conversation intact.

### Character-state updater (background "assistant")
- **CHAT-12** For characters that have both a persona profile (`extracted_context`) and a dynamic
  state (`character_state`) — i.e. world-extracted characters — the app runs a background
  **character-state update** after the character replies. It regenerates `character_state` from the
  recent conversation window (Soulfire-OG: last 10 messages) using the state/utility model (`AI-8`)
  at low temperature, treating the persona profile as immutable and producing an evolution (not a
  rewrite) of the dynamic state across emotional state, relationship with the player, current
  concerns, and unresolved threads.
- **CHAT-13** Character-state updates are **serialized and coalesced per character** (`AI-14`): only
  one update runs at a time per character; updates queued while one is running collapse to at most one
  pending run that re-reads fresh data. A failed update leaves the prior `character_state` intact.

### Lifecycle
- **CHAT-14** Deleting a chat removes its messages but keeps the character (`DATA-22`). The player can
  start a fresh chat with that character afterward.

## Acceptance criteria

- **AC-CHAT-a** (CHAT-1, CHAT-2) Opening a character with a Message-initial shows the verbatim
  opening with no model call; a Prompt-initial generates an opening; reopening the same character
  returns to the existing chat with history.
- **AC-CHAT-b** (CHAT-4, CHAT-6) A sent message appears instantly, the reply streams token-by-token,
  and the finalized reply persists; a simulated no-first-token timeout surfaces an error and saves
  nothing; a mid-reply stall saves the partial text.
- **AC-CHAT-c** (CHAT-8) A reply ending in an allowed emoji shows clean text and records that emoji as
  the character's reaction to the player's message.
- **AC-CHAT-d** (CHAT-9) A player reaction persists, displays under the message, and appears in the
  next prompt's history.
- **AC-CHAT-e** (CHAT-10, CHAT-11) After the configured number of messages, the summary updates and
  the counter resets; the chat remains usable throughout; a forced summary failure preserves the old
  summary.
- **AC-CHAT-f** (CHAT-12, CHAT-13) For a world-extracted character, the dynamic state changes after
  replies; rapid consecutive replies never run two state updates concurrently for that character.
- **AC-CHAT-g** (CHAT-3) The first exchange produces a short title; before that the default title is
  shown.

## Design notes (non-normative)

- Mirrors Soulfire-OG `services/chat/chat_service.rs` (`handle_chat_message`,
  `handle_ai_start_chat`, `generate_chat_summary`, `sanitise_message`) and
  `services/chat/character_state_service.rs` (`CharacterStateUpdater`). Constants to reuse:
  history=20, summary window=40, summary interval=20, state-update window=10, output floor=2000,
  stream idle timeout=30s, response timeout≈120s.
- In the local app these run as in-process async tasks driven by the UI rather than over a WebSocket;
  the streaming-delta and status model from Soulfire-OG (`ChatStatus` connecting/waiting/ready)
  becomes local UI state (`UI`).
- The character-state updater uses the cheap state/utility model at low temperature (≈0.3); the chat
  reply uses the chat/narrative model and the character's own sampling controls.
