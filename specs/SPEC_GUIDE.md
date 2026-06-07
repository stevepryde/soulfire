# How to write a spec

**Purpose:** the meta-spec. It defines what a specification is, the altitude to write it at, what it
must and must not contain, and how to tell when it is right. Read this before adding or editing any
file in `specs/`. When a `specs/README.md` exists, use it as the ownership map for how individual
specs divide up the system and which spec owns which facts.

## The core principle

A spec describes **what** the software does and **why**: its observable behavior and the contracts
others depend on, at a level that is:

1. **Precise enough** that a test author can validate it and an implementer can follow it *without
   reading the code*, and
2. **free enough of implementation detail** that the code can be rewritten: different functions,
   libraries, file layout, algorithms, or runtime internals, without the spec changing.

The spec is the benchmark. Tests prove the system matches the spec; code is judged against it. So the
spec must capture every behavior that matters, and nothing that does not.

## Three litmus tests

Apply these to every requirement:

- **Observability**: can you observe it at a boundary (an HTTP response, a WebSocket message, a UI
  behavior, a persisted state visible through the app, a shipped data file, a timing/limit, an error
  status)? If you cannot observe it, it probably is not a requirement; it is an implementation choice.
- **Testability**: could someone write a test from this sentence alone, without reading the code? If
  not, it is too vague or too abstract.
- **Implementation independence**: could two competent engineers satisfy this in different ways and
  both be correct? If your wording forbids that for no real reason, you have written *how*, not *what*.

A good requirement passes all three: observable, testable, and free to be implemented many ways.

## What a spec SHOULD contain

- **Observable behavior and outcomes**: given an input/action, what happens.
- **Interface contracts**: the things callers and users depend on, such as route paths and methods,
  message `type` names and payload fields, DTO field names, error shapes and status codes,
  environment variable names, configured limits, file/data formats, and UI affordances. These are
  precise *because they are the contract*.
- **Guarantees and invariants**: security, isolation, authority, data integrity, ordering/stability,
  idempotency, consistency, and single-source-of-truth rules.
- **Inputs, outputs, and error behavior**: what is accepted, what is rejected, and with what result.
- **Genuine decisions and constraints**: product/platform choices that are part of the product, with
  brief rationale for the non-obvious ones.
- **Acceptance criteria**: observable checks tied to requirement IDs.

## What a spec SHOULD NOT contain (in normative sections)

- Source code or pseudo-code.
- Function, method, or variable names; source file names; module/directory layout.
- Library-specific calls, flags, macro names, configuration syntax, or framework mechanics.
- Internal algorithms or data structures, unless the algorithm itself is the contract (for example, a
  wire format, a message ordering, a stable sort order, or a selection guarantee callers can observe).
- Specific dependency versions. Dependency manifests are the source of truth for versions.
- Step-by-step build instructions.

If an implementation detail is genuinely useful (a porting pointer, a known-good mechanism, a
compatibility warning), put it in a **Design notes** section and mark it non-normative: never put it in
Requirements or Acceptance criteria.

## Guarantee vs. mechanism: the most common mistake

State the **guarantee** (what must be true), not the **mechanism** (how you would achieve it).

| Too low (mechanism: avoid in requirements) | Right altitude (guarantee) |
|---|---|
| "Store session data in a `RwLock<HashMap>` keyed by ID." | "Session state is held server-side and is the single source of truth for all connected clients." |
| "Broadcast updates over a `tokio::sync::broadcast` channel." | "Every accepted state change is delivered to all connected clients authorized to observe it." |
| "Deserialize each frame with `serde_json::from_str`." | "Messages are JSON objects with a `type` field; malformed or unrecognized messages fail without corrupting state." |
| "Call `safe_ident(raw, idx)` to sanitize names." | "User-provided identifiers are normalized to a safe, unique, stable form before they are exposed through the contract." |
| "Wrap the query in `LIMIT ? OFFSET ?`." | "Results are paginated and stable: the same request returns consistent, non-overlapping pages." |
| "Spawn a task per connection." | (omit: not observable; it is an implementation concern) |

The right-hand column is testable and survives a rewrite. The left-hand column belongs in Design
notes, or nowhere.

## Spec file structure

Every spec file contains, in this order:

1. **Purpose**: one line.
2. **Requirements**: normative. Each is one testable statement with a stable ID (`PREFIX-n`). Use
   "must" language; avoid "should/may" here. Those belong in Design notes.
3. **Acceptance criteria**: observable checks that prove the requirements; each references the
   requirement ID(s) it covers.
4. **Design notes** *(optional, non-normative)*: rationale, tradeoffs, implementation hints, migration
   notes, and useful references. Clearly mark this section as non-authoritative: code may ignore
   these notes as long as it satisfies the Requirements.

Facts live in exactly one spec; cross-link by ID instead of duplicating.

## Calibrating detail by spec type

- **Interface specs**: precise paths, message types, field names, statuses, file formats, and state
  transitions *are* the contract; include them exactly.
- **Behavior specs**: observable outcomes and guarantees; keep internals out.
- **Data specs**: persisted/shipped data formats, identity rules, lifecycle guarantees, retention, and
  content constraints.
- **Posture specs**: security, privacy, reliability, performance, operational, and abuse-resistance
  guarantees plus the observable values a user, test, or tool can check.
- **UI specs**: the screen-flow behavior and the visual/interaction contract (tokens, scales,
  components, accessibility, and input behavior) the client must honor.
- **Process specs**: test strategy, traceability, release readiness, deployment, and CI outcomes;
  concrete tooling and commands stay in Design notes.

## Review checklist

Before committing a spec change, confirm:

- [ ] Every requirement is observable and testable.
- [ ] No code, function names, source file paths, library calls, or version numbers appear in
      normative sections.
- [ ] The implementation could vary; the spec does not over-constrain *how*.
- [ ] Every acceptance criterion references a requirement ID.
- [ ] No fact is duplicated from another spec; cross-link instead.
- [ ] Non-obvious decisions carry a brief rationale.
- [ ] A test author could write tests from this file alone.
- [ ] Implementation hints, if any, are confined to a non-normative Design notes section.
