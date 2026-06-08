# Manual Smoke Checklist

Status: current Tauri/React shell checklist. Update this file as new OG parity surfaces land.

## Before Smoke

- Run `bun run build`.
- Run `cargo build --workspace --locked`.
- Run `cargo test --workspace --locked` before release-bound changes.
- Start the shell with `bun run dev` for browser-preview checks, or `bun run tauri dev` once native launch smoke is the target.

## Store Setup And Unlock

- Open the app shell.
- On a fresh data folder, Setup is selected and Unlock is disabled.
- Creating a store with a non-empty master password unlocks the app.
- Locking/reopening requires the same master password.
- A wrong password leaves the app locked and shows an error without exposing internal storage details.

## Worlds Shell

- Worlds loads in-progress adventures and world blueprints through Tauri commands after unlock.
- The world search field filters blueprints and does not affect active adventures.
- Load More appears only when the blueprint list has another cursor page.
- Selecting a blueprint opens a read-only detail panel with its description and world prompt.
- Selecting an active adventure opens a read-only state panel and can show the next-turn prompt view.

## Characters Shell

- Characters loads through the Tauri list command after unlock.
- Search filters characters and Load More advances cursor pages without duplicating rows.
- Selecting a character opens a read-only detail panel.
- Prompt View shows section headers, locked/editable labels, sources, and token estimates.
- Saving an editable character prompt section updates the detail prompt and keeps locked sections read-only.

## Settings Shell

- Store status, schema version, runtime, accent, and adult-content state render after unlock.
- OpenAI key status never reveals the full key; save/delete only show masked status.
- Accent swatches save through `save_app_settings`; no native select appears.
- Adult Content toggles through `save_app_settings`.
- Player profile fields save through `save_player_profile`.

## Token Stats

- Stats renders aggregate request/input/cached-output totals.
- Model and operation breakdowns show token totals without cost estimates.
- Clear History opens an in-app confirmation modal.
- Cancel leaves stats unchanged; Clear removes local usage metrics only.

## Guardrails

- No native browser `confirm()` / `prompt()` appears for destructive actions.
- No native HTML `<select>` appears in app UI.
- No API key is written to browser storage, logs, or visible unmasked text.
- Browser preview failures caused by unavailable Tauri runtime are shown as app errors, not crashes.
