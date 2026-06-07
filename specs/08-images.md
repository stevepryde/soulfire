# Images

**Purpose:** define AI-generated character portraits and world cover images, local storage of image
bytes, optional local upload, and the crop/transform editors. Entities are owned by `DATA`; the
provider call by `AI`; at-rest protection by `SEC`; screens by `UI`.

## Requirements

### Generation
- **IMG-1** The user can generate an **AI portrait** for a character and an **AI cover** for a world
  using their configured provider key (`AI-3`). Generation derives its prompt from the entity (name,
  description, prompt/world content) and produces an image stored locally.
- **IMG-2** Generation runs asynchronously and does not block other interaction; while a generation is
  in progress the entity shows an in-progress indicator, and on completion the new image appears
  wherever that entity is rendered. A failed generation surfaces an error and leaves any prior image
  in place.
- **IMG-3** The user can **regenerate** an entity's image (replacing the current one) and can clear it
  back to the emoji avatar/cover (`DATA-20`). Each generation is metered like any AI call
  (`AI-15`, label `image_generation`).

### Storage & protection
- **IMG-4** Generated image bytes are stored locally and protected at rest (`SEC-3`): inside the
  encrypted store, or as files encrypted under a key held only in the encrypted store. An entity
  references its image plus a version/cache-bust value so updated images re-render.
- **IMG-5** No image is uploaded anywhere except as part of the provider generation request; images
  are never sent to any other service.

### Local upload
- **IMG-6** The user can set a character portrait or world cover from a **local image file** they
  choose. Uploaded images are stored and protected identically to generated ones (IMG-4) and support
  the same framing editor (IMG-7).

### Rendering precedence & framing
- **IMG-7** When an entity has a stored image (generated or uploaded), it is rendered with the
  entity's stored **transform** (pan x/y percent, zoom percent). The character editor frames within a
  **round** crop (zoom up to ~240%); the world editor frames within a **16:6** rectangle (zoom up to
  ~220%). Both editors support drag-to-pan, a zoom control, and a reset-framing action, on pointer and
  touch input. The transform persists with the entity (`DATA-1`/`DATA-8`).
- **IMG-8** Rendering precedence matches Soulfire-OG: a stored image (with transform) takes precedence
  over the entity's emoji selection, which takes precedence over a bundled default. World cover
  rendering falls back to a large centered emoji when no image is stored.

## Acceptance criteria

- **AC-IMG-a** (IMG-1, IMG-2) Generating a portrait shows an in-progress state then renders the image
  on the character everywhere; a forced failure shows an error and keeps the prior image.
- **AC-IMG-b** (IMG-3) Regenerating replaces the image and bumps its version so the UI updates;
  clearing returns to the emoji avatar.
- **AC-IMG-c** (IMG-4, SEC-3) A generated/uploaded image is unreadable from disk without the app's
  key.
- **AC-IMG-d** (IMG-6) A user-selected local file becomes the portrait/cover and is framable.
- **AC-IMG-e** (IMG-7, IMG-8) The crop editors pan/zoom/reset on mouse and touch and persist the
  transform; precedence (stored image → emoji → default) holds.

## Design notes (non-normative)

- Mirrors Soulfire-OG's image features (`CharacterPortraitTransformEditor`, `WorldCoverTransformEditor`,
  `CharacterPicture`/`WorldPicture`/`WorldCoverMedia`, the image-generation services and websocket
  "image ready" notifications) but replaces R2 object storage with local encrypted storage and
  replaces the websocket-ready signal with a local task-completion signal.
- OpenAI image generation (e.g. the image model) satisfies IMG-1 under BYOK; store returned bytes
  (PNG/WebP) locally. Keep the generation prompt builder small and separate so a future provider can
  supply images without touching the editors.
- IMG-6 (local upload) is a new local-only capability beyond Soulfire-OG; it reuses the same storage
  and framing path as generated images.
