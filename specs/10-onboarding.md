# Onboarding (First Run)

**Purpose:** define the first-run experience, the bundled starter worlds, and deferred player
customization. Adapts Soulfire-OG's "story, not interface" first-run design to a local, single-user,
BYOK app. Unlock/security is owned by `SEC`; screens by `UI`; worlds by `WORLD`.

## Requirements

### First-run sequencing
- **ONB-1** On the very first launch the user completes a minimal setup before any AI feature: set the
  **master password** (`SEC-4`) and add an **OpenAI API key** (`AI-3`). These are the only required
  steps; everything else is deferred. The app states plainly that there is no password recovery
  (`SEC-4`). The setup also asks whether to remember unlock on this device (`SEC-7`), using the
  platform-specific default toggle state from `SEC-7`. Biometric unlock gating (`SEC-13`) is not part
  of required first-run setup for the first release.
- **ONB-2** After setup, first-time users are taken **straight into a story**, not the home screen,
  reproducing Soulfire-OG's principle that the first experience is story rather than interface. The
  app seeds the bundled starter worlds (ONB-5), picks a strong starter world, captures the player's
  name with a single low-friction prompt (skippable), and launches an adventure directly.
- **ONB-3** The name capture is one screen over an atmospheric dark background: a serif prompt
  ("What shall we call you?"), a single text field (prefilled from the profile nickname if any), a
  Continue action (saves the name to the PlayerProfile, `DATA-17`), and a Skip action (launches with a
  generated default name). Submitting either launches the chosen world's intro (`UI-13`, `WORLD-3`).
- **ONB-4** A first-time user can opt out of the auto-started world via a subtle exit (e.g. "Browse
  other worlds") that lands them on the worlds home; first-run is marked complete either way so the
  auto-start does not recur. If no starter world is available, the user is taken to the home screen
  with a friendly message.

### Bundled starter worlds
- **ONB-5** The app ships a small curated set of **starter worlds** as ordinary editable blueprints
  (`DATA-21`), seeded on first launch. They are chosen for strong cold opens (Soulfire-OG lead
  example: "Beneath Verath"). After seeding they are fully user-editable and deletable; re-seeding does
  not duplicate or resurrect deleted starters. Each starter has a stable seed id; the seed ledger
  (`DATA-24`) records which blueprint was created for that starter and whether the user deleted it.
  The lead starter is `beneath_verath` ("Beneath Verath").

### Returning users & deferred customization
- **ONB-6** After first run, the app opens to the **worlds home** (`UI-8`). When an in-progress
  adventure exists, "continue where you left off" is the dominant action; when none exists, "start a
  new adventure" / browse worlds is offered. There is no welcome banner.
- **ONB-7** Player customization (name, attributes, prompt extension) is **not** required up front; it
  lives in **Adventure Defaults** in settings (`UI-20`, `DATA-17`) and only affects adventures started
  afterward (`WORLD-3`). The app may prompt once, after the first session, to set an adventurer name
  for next time.

## Acceptance criteria

- **AC-ONB-a** (ONB-1) A brand-new install requires setting a master password and adding an OpenAI key
  before any AI action, warns that the password is unrecoverable, and asks whether to remember unlock
  on this device with the correct platform default.
- **AC-ONB-b** (ONB-2, ONB-3) After setup, the first-time user sees the name prompt over an
  atmospheric background and is dropped directly into a starter world's intro; Skip launches with a
  default name.
- **AC-ONB-c** (ONB-4) Choosing the subtle exit lands on the worlds home and the auto-start does not
  happen again on next launch.
- **AC-ONB-d** (ONB-5) Starter worlds appear as editable blueprints on first launch with stable seed
  ids recorded in the ledger; deleting one and relaunching does not bring it back or create
  duplicates.
- **AC-ONB-e** (ONB-6) A returning user with an in-progress adventure sees "continue" as the primary
  action; with none, sees "start a new adventure".

## Design notes (non-normative)

- Mirrors Soulfire-OG `pages/first_run.rs` and `manifest/first-run-flow.md`, with two adaptations for
  the local app: (1) the account-creation step is replaced by the local setup (master password + API
  key, ONB-1); (2) the "featured worlds" source becomes the bundled starter blueprints rather than a
  server catalog.
- Ship the starter worlds as data files seeded into the store on first launch; track a "seeded" marker
  and per-starter seed ids so deletions are respected (ONB-5).
- Keep the brand voice from Soulfire-OG's landing/onboarding copy (immersive, self-discovery framing)
  for the setup and intro screens (`PROD` design notes).
