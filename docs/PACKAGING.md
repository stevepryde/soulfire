# Packaging & platforms

How Soulfire builds and ships across its five targets, and where it keeps data
(`specs/12-platform-packaging.md`). The same `app` crate builds every platform
from one source tree (`PKG-1`); the pure engine lives in `soulfire-core`.

## Data location (PKG-3)

All persistent state lives in a single per-user application-data directory: the
encrypted database `soulfire.db` and the plaintext key-derivation sidecar
`soulfire.meta.json` (the sidecar holds only the salt, KDF parameters, and a
one-way verifier — no secrets). Generated/uploaded image bytes live **inside**
the encrypted database, so there are no plaintext media files on disk (`SEC-3`).

The directory is resolved via the platform dirs (`directories::ProjectDirs`,
qualifier `com`, org/app `Soulfire`):

| Platform | Location |
|----------|----------|
| Linux    | `$XDG_DATA_HOME/Soulfire` (default `~/.local/share/Soulfire`) |
| macOS    | `~/Library/Application Support/com.Soulfire.Soulfire` |
| Windows  | `%APPDATA%\Soulfire\Soulfire\data` |
| Android  | the app sandbox's data dir |
| iOS      | the app sandbox's Application Support dir |

Uninstalling the app does not delete this directory except where the platform
mandates it. A newer build opens an older store and migrates it forward
transparently via SQLite's `user_version` and the per-record `version` field
(`PKG-4`); the build embeds no API key or secret (`PKG-5`).

## Building & running

```sh
# Dev (any desktop OS):
cd app && dx serve --platform desktop

# Release bundles:
cd app && dx bundle --platform desktop        # Win/macOS/Linux installer
cd app && dx bundle --platform android        # APK/AAB
cd app && dx bundle --platform ios            # .app / .ipa

# Plain cargo (compiles the desktop app without dx):
cargo build -p soulfire --release
```

SQLCipher is built from vendored sources (`rusqlite` `bundled-sqlcipher-
vendored-openssl`), so no system crypto library is required; Linux needs the
usual WebKitGTK/GTK dev packages for the desktop webview (see `.github/workflows/ci.yml`).

## Styles & fonts

The compiled Tailwind stylesheet `app/assets/tailwind.css` is committed so the
app builds and styles without a Tailwind step. Regenerate it after changing
classes:

```sh
npx @tailwindcss/cli -i app/assets/input.css -o app/assets/tailwind.css
```

Fonts: the UI calls for **Inter** (sans) and **Merriweather** (serif). To bundle
them offline-first (no CDN, `SEC-11`), drop the OFL-licensed `.woff2` files under
`app/assets/fonts/` and add `@font-face` rules to `input.css`; until then the CSS
falls back to the platform sans/serif. (Pending — see `docs/BUILD_PLAN.md`.)

## Release outcomes (PKG-5/6)

- A clean build produces a runnable artifact per target containing no embedded
  secret. Signing/notarization (macOS, store signing) are configured per release
  rather than in the default build.
- The repository ships both `LICENSE-MIT` and `LICENSE-APACHE` and declares
  `MIT OR Apache-2.0` in package metadata and the README (`PROD-16`).
