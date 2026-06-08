# AGENTS.md — how we build Soulfire

Soulfire is **spec-driven**. The specifications in [`specs/`](specs/) — together with the UI design
language they carry ([`specs/09-ui.md`](specs/09-ui.md)) — are the single source of truth for what
this software is and how it behaves. **Code implements the spec; tests validate the code against the
spec.** When the three disagree, **the spec wins** — you change the code or the test, not the spec,
unless you are deliberately changing the design (in which case you change the spec *first*).

The North Star and scope live in [`specs/00-product-overview.md`](specs/00-product-overview.md) —
decide by it when the path is unclear. **Security and the user's data privacy come first and are never
traded away** (see [`specs/02-storage-security.md`](specs/02-storage-security.md)); correctness,
faithful reproduction of Soulfire-OG, usability, maintainability, and keeping spec/tests current
follow. A change should advance the goals and regress none — above all security and privacy.

## The three artifacts and their jobs

| Artifact | Question it answers | Authority |
|----------|--------------------|-----------|
| **Spec** (`specs/*.md`, incl. the UI design language) | *What* should it do, and *why*? | Source of truth |
| **Tests** (unit/integration, manual smoke steps) | Does the code actually do what the spec says? | Validation against the spec |
| **Code** (Rust core + Tauri/React app) | *How* is it done? | Implementation of the spec |

A behavior that is not in the spec does not exist. Code exists to satisfy the spec; a test exists to
prove a specific spec requirement once the behavior has settled.

## The loop — every change follows this order

This is **spec-first, not test-first** (not TDD). Building is partly exploration: you rarely know the
final shape up front, and discoveries during implementation feed back into the spec. Tests come
*after* the behavior settles, to lock in what the spec says.

1. **Spec first.** Before writing or changing code, update the relevant spec file(s) in `specs/`.
   - New behavior → add/extend requirements (with IDs) and acceptance criteria.
   - Changed behavior → edit the requirement and note the change.
   - Removed behavior → delete the requirement and any tests/code that only existed for it.
   - If a change spans domains, update each affected spec and keep cross-links (`[CHAT-4](specs/05-chat.md)`)
     consistent. The spec at this point is the intended design — a hypothesis, not a frozen contract.
2. **Implement (explore).** Build it. The implementation answers *how*; it must not introduce behavior
   the spec doesn't describe. **When implementation reveals the spec is wrong, incomplete, or the wrong
   shape — normal and expected — go back and update the spec** so it matches the design you've
   discovered. The spec and code converge before you write tests.
3. **Then write tests.** Once the behavior has settled, write tests that validate the spec against the
   code, per [`specs/13-testing.md`](specs/13-testing.md). Each test names the requirement ID(s) it
   proves. Aim for every requirement to have a test or a documented manual smoke step.
4. **Verify.** Run the automated checks and confirm the smoke steps. A change is done when the spec,
   code, and tests agree.

> The spec is the source of truth, but it's a *living* one: it leads each change and is corrected by
> what building teaches. Never let code drift from the spec silently. The order is spec → code → tests;
> the rule is that all three agree before a change is done.

## Requirement IDs

Normative requirements carry a stable ID: `<PREFIX>-<n>` (e.g. `DATA-1`, `WORLD-12`, `SEC-7`). Prefixes
map to spec files — see [`specs/README.md`](specs/README.md) for the registry. Rules:

- IDs are **stable and append-only**. Never renumber. If a requirement is removed, retire its ID
  (`~~SEC-7~~ (removed YYYY-MM-DD)`) rather than reusing the number.
- Each requirement is a single, testable statement.
- Acceptance criteria and tests reference requirement IDs so coverage is traceable both ways:
  spec → test (every requirement has a test or a documented manual smoke step) and test → spec (every
  test names what it proves).

## Spec conventions

**[`specs/SPEC_GUIDE.md`](specs/SPEC_GUIDE.md) is the authority on how to write a spec** — read it
before authoring or editing one. In brief: specs describe **observable behavior and contracts**
(what/why), at an altitude that is testable yet free of implementation detail. **No code, function
names, file paths, library calls, or version numbers in normative sections** — those go in
non-normative *Design notes*. One domain per file; facts live in exactly one place; cross-link instead
of duplicating. [`specs/README.md`](specs/README.md) is the index and ID registry — read it first.

## Project-specific guardrails

- **Single-user, local, BYOK.** No accounts, no auth, no billing, no server, no remote store. All data
  is on-device; the only network actions are calls to the user's own AI provider
  ([`specs/00-product-overview.md`](specs/00-product-overview.md), [`specs/03-ai-integration.md`](specs/03-ai-integration.md)).
- **Encrypted at rest; keys never leak.** The whole store is encrypted under the user's master
  password; API keys live only inside it and never appear in logs, errors, or UI unmasked
  ([`specs/02-storage-security.md`](specs/02-storage-security.md)). Privacy and at-rest encryption are
  never optional.
- **Faithful to Soulfire-OG.** Soulfire-OG (the online app at `~/projects/app-world/soulfire`) is the
  **behavioral reference**: prompt assembly, the turn engine, memory cadences, and the look/feel are
  reproduced exactly except where a spec deliberately changes them. Do not invent behavior OG lacks
  (e.g. unimplemented "chat modes", `PROD-14`); do not reintroduce its accounts/billing/admin/
  moderation/ratings/public-content framing (`PROD-11`).
- **Async app bridge.** Tauri command handlers that touch storage, AI, image generation, import/export,
  or any other potentially blocking work must be `async` and must enter encrypted storage through the
  core async store facade. Do not call the synchronous `Store` directly from app-shell commands; keep
  it inside core internals, blocking workers, and focused tests.
- **No cut corners.** Build the full-featured version, not an MVP. If something is under-specified,
  fix the spec — don't ship a stub.
- **Native/local app.** The approved direction is Tauri v2 + React with Rust owning durable product
  logic, storage, AI orchestration, prompts, and security-sensitive behavior. The next implementation
  step is to update the specs for this stack pivot before rebuilding the UI.
- **Dual licensed** `MIT OR Apache-2.0` (`PROD-16`).
