# Platform & Packaging

**Purpose:** define target platforms, where data lives, and the build/release outcomes. Storage
mechanics are owned by `SEC`; licensing posture by `PROD`.

## Requirements

### Targets
- **PKG-1** Soulfire builds and runs as a native application on **Windows, macOS, Linux, Android, and
  iOS** from a single Rust + Dioxus codebase (`PROD-3`). Each platform launches to the lock/setup
  screen and is fully functional (subject to a configured API key for AI features).
- **PKG-2** The UI adapts to each platform's form factor (`UI-3`): desktop window resizing and the
  sidebar nav; mobile touch, safe areas, and the bottom nav. There is no functional divergence between
  platforms beyond input/layout adaptation and platform-secure-store availability (`SEC-7`).

### Data location & portability
- **PKG-3** The encrypted store and any encrypted image files live in a single per-user
  application-data location appropriate to each platform; uninstalling the app does not require the
  user's data to be deleted unless the platform mandates it. The data location is discoverable to the
  user (shown in settings or docs).
- **PKG-4** A future store-format migration is supported by the per-record `version` (`DATA`) and the
  versioned KDF/verifier (`SEC`); upgrading the app must not lose or corrupt existing data, and a
  newer app opening an older store migrates it forward transparently.

### Build & release outcomes
- **PKG-5** A documented, reproducible build produces a runnable artifact per target platform. The
  build does not embed any API key or secret. Release artifacts are unsigned-by-default but the
  process accommodates platform signing where required (notarization, store signing).
- **PKG-6** The repository ships the dual-license texts and declares `MIT OR Apache-2.0` in package
  metadata and README (`PROD-16`).

## Acceptance criteria

- **AC-PKG-a** (PKG-1, PKG-2) The same source tree builds and launches on all five targets; each
  reaches the lock/setup screen and, with a key, completes a chat turn and an adventure turn.
- **AC-PKG-b** (PKG-3) The app reads and writes its store at the documented per-platform location;
  the location is surfaced to the user.
- **AC-PKG-c** (PKG-4) Installing a newer build over an older store opens existing chats, characters,
  worlds, and adventures without loss.
- **AC-PKG-d** (PKG-5, PKG-6) A clean build yields per-platform artifacts containing no embedded
  secret, and the repo declares the dual license with both texts present.

## Design notes (non-normative)

- Dioxus desktop (via its native renderer) covers Windows/macOS/Linux; Dioxus mobile covers
  Android/iOS. A small workspace with a shared core crate (models, store, AI, prompt/turn engines)
  and a thin per-platform UI shell keeps PKG-2 cheap. `rusqlite` with bundled SQLCipher links the
  encrypted store on every target (`SEC`).
- Recommended data locations: per-OS app-data/config dirs (e.g. a platform dirs crate). On mobile the
  app sandbox is the natural location; on desktop a clearly named app folder under the user profile.
- CI should build all five targets; signing/notarization are environment-specific and configured per
  release rather than in the default build.
- Porting note: Soulfire-OG's web build (`Dioxus.toml` web platform, service worker, PWA manifest) is
  replaced by desktop/mobile bundles; keep the asset pipeline (Tailwind build, bundled fonts/icons/
  character art) but drop web-only files.
