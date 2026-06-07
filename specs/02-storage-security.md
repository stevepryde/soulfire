# Storage & Security

**Purpose:** define where local state lives, how it is encrypted at rest, and how the app is
unlocked. Field-level data shapes are owned by `DATA`; credential usage is owned by `AI`.

## Requirements

### Encrypted store
- **SEC-1** All persistent application data (`DATA` entities, settings, and credentials) is stored in
  a single local database that is **encrypted at rest**. With the database file at rest and the app
  not running, no application content (chats, characters, worlds, prompts, API keys) is readable from
  the file without the unlock secret.
- **SEC-2** Encryption is whole-database and transparent to feature code: enabling it changes no
  feature behavior and loses no functionality. The encryption key is derived from the user's master
  password (SEC-5) via a slow, salted key-derivation function with parameters chosen to be
  expensive to brute-force on commodity hardware.
- **SEC-3** Generated image bytes (`IMG`) are also protected at rest: either stored inside the
  encrypted database, or stored as files encrypted with a key held only in the encrypted database.
  No portrait/cover image is readable from disk without the unlock secret.

### Master password & unlock
- **SEC-4** On **first launch**, the app guides the user to set a **master password**. The password
  is never stored; only material needed to verify it and derive the key (e.g. a salt and a
  verifier) is stored. There is no recovery mechanism if the password is lost, and the app states
  this plainly before the password is set.
- **SEC-5** On **each launch**, the app is **locked** until the correct master password is supplied
  (or the device-remembered key is used, SEC-7). While locked, no `DATA` content and no credentials
  are accessible, and no AI calls can be made.
- **SEC-6** An incorrect master password is rejected without revealing whether any particular field
  is correct, and without unlocking any data. Repeated wrong attempts do not corrupt the store.
- **SEC-7** The user may opt in, per device, to **remember the unlock on this device**. When enabled,
  the key (or a wrapping secret for it) is stored in the operating system's secure credential store
  (platform keychain/keystore), and subsequent launches on that device unlock without prompting.
  Disabling the option, or the OS store becoming unavailable, falls back to the master-password
  prompt. The remembered secret is removed when the user disables the option.
- **SEC-8** The app provides a **lock** action that returns to the locked state without quitting, and
  a **change master password** action that re-keys the store (re-deriving/ re-wrapping the
  encryption key) such that the old password no longer unlocks it. Changing the password updates any
  device-remembered secret accordingly.

### BYOK credential handling
- **SEC-9** Provider API keys (`DATA-19`) are stored only inside the encrypted store and are
  decrypted into memory only to make provider requests. They are never written to logs, telemetry,
  crash reports, error messages, or any unencrypted file.
- **SEC-10** After entry, an API key is displayed masked (e.g. last few characters only). The user
  can replace or clear a key but is never shown the full stored value again through normal UI.

### Data boundaries
- **SEC-11** No application data is transmitted anywhere except to the user's configured AI provider
  as part of fulfilling an explicit AI action (a chat turn, an adventure turn, a build step, an image
  generation, or token counting). The app performs no background analytics or telemetry network
  calls.
- **SEC-12** Export/backup, if offered, produces an artifact that is itself encrypted or explicitly
  warns that it is unencrypted before writing; the default path never silently writes plaintext
  application data to disk.

## Acceptance criteria

- **AC-SEC-a** (SEC-1, SEC-2) Inspecting the database file with the app closed and without the
  password reveals no readable chat text, character prompts, world prompts, or API keys; with the
  correct password the same data loads normally.
- **AC-SEC-b** (SEC-4, SEC-5, SEC-6) First launch requires setting a password; later launches require
  it (or the remembered key); a wrong password leaves the app locked with all data inaccessible and
  the store intact.
- **AC-SEC-c** (SEC-7) With "remember on this device" enabled, relaunch unlocks without a prompt;
  disabling it removes the stored secret and the next launch prompts again.
- **AC-SEC-d** (SEC-8) After "change master password", the old password fails and the new one
  succeeds; a device-remembered unlock still works post-change.
- **AC-SEC-e** (SEC-9, SEC-10) Triggering an error during a provider call produces no log/UI output
  containing the API key; the settings screen shows the key only masked.
- **AC-SEC-f** (SEC-3) Generated image files (if stored as files) are unreadable as images without
  the app's key.

## Design notes (non-normative)

- A SQLCipher-backed `rusqlite` (the `bundled-sqlcipher` feature) gives transparent whole-database
  AES encryption satisfying SEC-1/SEC-2 with no changes to query code. The user's master password,
  stretched through a strong KDF (e.g. Argon2id) into the SQLCipher key, keeps key derivation in app
  control and lets SEC-8 re-key via `PRAGMA rekey`.
- For SEC-7, store either the derived key or a random wrapping key in the OS store via a
  cross-platform keychain crate (macOS Keychain, Windows Credential Manager/DPAPI, Linux Secret
  Service, iOS Keychain, Android Keystore). On platforms without a usable secure store, SEC-7 simply
  stays unavailable and the app falls back to the password prompt.
- Mobile keyboards and biometric prompts can front the OS keychain unlock (e.g. Face ID gating
  retrieval of the remembered key); this is an enhancement of SEC-7, not a separate requirement.
- KDF parameters and the verifier scheme should themselves be versioned so they can be strengthened
  by a future migration.
