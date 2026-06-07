# Manual smoke checklist

The native UI's **visual and interaction contract** (`specs/09-ui.md`) is verified
by this per-platform manual checklist, because Dioxus desktop/mobile has no mature
end-to-end driver (`TEST-6`). The pure logic behind the screens is unit/integration
tested in `soulfire-core`; this list covers what only a human can see.

## Running

```sh
cd app
# Desktop (Windows / macOS / Linux):
dx serve --platform desktop
# Mobile:
dx serve --platform android   # or: ios
```

The bundled `app/assets/tailwind.css` is committed, so the app builds and styles
without a Tailwind step. After changing CSS classes, regenerate it:

```sh
# Tailwind v4 CLI, scanning src + the vendored sp-ui/sp-markdown class sources:
npx @tailwindcss/cli -i app/assets/input.css -o app/assets/tailwind.css
```

## Checklist (run per platform)

### First run & unlock (`SEC`, `ONB`, `UI-23`)
- [ ] First launch shows the set-password screen with the "no recovery" warning.
- [ ] Setting a password (+ optional OpenAI key) enters the app; starter world
      "Beneath Verath" appears under Worlds.
- [ ] Relaunch shows the unlock screen; wrong password is rejected; correct opens.
- [ ] Profile → "Lock app" returns to the unlock screen without quitting.

### Theme & shell (`UI-1`..`UI-6`)
- [ ] Settings → each of the 7 accent colors recolors the wordmark, buttons, and
      spinners app-wide and persists across relaunch. Background stays dark.
- [ ] Desktop shows the left sidebar; narrow window / mobile shows the bottom nav;
      the active destination is indicated.
- [ ] Standard pages show the title bar; Play and Chat hide all chrome (immersive).
- [ ] At 320px width nothing scrolls horizontally; tap targets are ≥44px.

### Worlds & play (`UI-8`..`UI-13`, `WORLD`)
- [ ] Worlds home has Adventures and Worlds tabs with cards (16:6 cover + emoji).
- [ ] "Enter World" starts an adventure and drops into the immersive Play screen.
- [ ] Narration renders as large serif prose and streams token-by-token; the player
      action shows as a right "YOU" bubble; the composer status line updates.
- [ ] `/gm skip to morning` produces a staged proposal with a before/after diff and
      Accept/Reject; Accept applies it, Reject leaves state unchanged.
- [ ] `/gm` with no text and an unknown `/x` command each show a warning toast.
- [ ] Deleting an adventure / world asks for confirmation (world requires typing
      "delete"); after delete it disappears.

### Characters & chat (`UI-14`..`UI-17`, `CHAT`)
- [ ] Characters list shows avatar/name/subtitle with New Character + Edit/Delete.
- [ ] Opening a character enters the immersive chat; the opening message appears.
- [ ] Sending a message streams the reply token-by-token; bubbles render markdown.
- [ ] Tapping a bubble shows the allowed-emoji picker; choosing one shows the
      reaction under the bubble.

### Editors & prompt viewer (`UI-18`, `PROMPT`)
- [ ] Character editor saves name/subtitle/description/prompt/initial message and
      creativity; empty name/prompt/initial is rejected with a clear message.
- [ ] "View system prompt" lists the assembled sections in order, each labeled
      Locked or Editable with an estimated token count and a total.
- [ ] Editing the Character Prompt section in the viewer and saving changes the
      same field shown in the editor.
- [ ] With Adult content off (Settings → Content) the mature-roleplay section is
      absent from the prompt; turning it on adds it.

### Settings & stats (`UI-20`, `STAT`)
- [ ] API key shows masked (last 4 only); Replace updates it.
- [ ] Token Statistics shows totals, by-model and by-operation breakdowns, no cost
      figure; Clear empties them.
```
