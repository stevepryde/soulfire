# Soulfire (local) — Specs Index & Ownership Map

This directory specifies **Soulfire**, a single-user, local, BYOK desktop/mobile rewrite of the
online app referred to here as **Soulfire-OG** (reference source at `~/projects/app-world/soulfire`).
The product, design, and feature set are reproduced faithfully from Soulfire-OG, **minus** user
accounts, authentication, billing, moderation, ratings, public/shared content, and admin — plus a
small set of new local-only features.

Read [SPEC_GUIDE.md](SPEC_GUIDE.md) before editing any spec. Facts live in exactly one spec; other
specs cross-link by requirement ID (`PREFIX-n`).

## Ownership map

| Spec | Prefix | Owns |
| --- | --- | --- |
| [00-product-overview.md](00-product-overview.md) | `PROD` | Vision, platforms, scope, non-goals, what is dropped vs Soulfire-OG, licensing |
| [01-data-model.md](01-data-model.md) | `DATA` | All persisted entities, fields, IDs, formats, relationships, validation |
| [02-storage-security.md](02-storage-security.md) | `SEC` | Encrypted local store, master password + keychain unlock, BYOK key storage |
| [03-ai-integration.md](03-ai-integration.md) | `AI` | AI provider abstraction, OpenAI (BYOK), streaming, structured output, model registry, request metering |
| [04-system-prompts.md](04-system-prompts.md) | `PROMPT` | Prompt assembly, locked vs editable sections, content toggles (incl. adult content), prompt-viewer/editor feature |
| [05-chat.md](05-chat.md) | `CHAT` | Character chat behavior, streaming, reactions, rolling summary, character-state updater |
| [06-characters.md](06-characters.md) | `CHAR` | Character model surface, manual editor, conversational builder, NPC extraction from worlds |
| [07-worlds.md](07-worlds.md) | `WORLD` | Blueprints, adventures, turn engine, adventure-state schema, memory ladder, validator, `/gm` commands, world builder |
| [08-images.md](08-images.md) | `IMG` | AI image generation (portraits/covers), local image storage, upload, crop/transform editors |
| [09-ui.md](09-ui.md) | `UI` | Screen flow, routing, app shell, theme system, design tokens, visual/interaction contract |
| [10-onboarding.md](10-onboarding.md) | `ONB` | First-run flow, starter worlds, deferred player customization |
| [11-token-stats.md](11-token-stats.md) | `STAT` | Token statistics feature: capture, aggregation, surfaces |
| [12-platform-packaging.md](12-platform-packaging.md) | `PKG` | Target platforms, packaging, data locations, build/release outcomes |
| [13-testing.md](13-testing.md) | `TEST` | Test strategy, testability seams, coverage, traceability — how tests validate the code against the spec |

## Reading order for implementers

1. `PROD` (what we are building and not building)
2. `SEC` + `DATA` (where state lives and what it looks like)
3. `AI` + `PROMPT` (the model layer and prompt contracts)
4. `CHAT`, `CHAR`, `WORLD`, `IMG` (the three feature pillars + media)
5. `UI` + `ONB` + `STAT` (the surface)
6. `PKG` (shipping)
7. `TEST` (how all of the above is validated)

## Source-of-truth conventions

- Soulfire-OG is the **behavioral reference**. Where this spec quotes prompt text or names a contract
  field, that text/field is taken from Soulfire-OG and is normative for the rebuild unless a
  requirement explicitly changes it.
- Requirements that say to reproduce Soulfire-OG are intentional references to the OG source, not
  permission to build a reduced summary implementation. Implementers inspect and port the relevant OG
  behavior directly, trimming only the systems this spec explicitly removes.
- Soulfire-OG persisted "record" shapes are the basis for `DATA`; the rebuild keeps the same field
  names and semantics so prompts and behavior port unchanged, dropping account/billing/moderation
  fields.
