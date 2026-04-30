# Dev Notes Layout

This directory is split by responsibility.

## Directories

- `v1/`
  - Notes that are already covered by the Favnir v1.0.0 roadmap or by the versions directly leading into it.
- `post-v1/`
  - Language/runtime/tooling ideas that are intentionally deferred until after v1.0.0.
- `veltra/`
  - Product/platform notes for Veltra, including notebook and product-facing ideas.

## Intended Use

Use `v1/` when implementing or reviewing the v1 language/runtime/tooling scope.

Use `post-v1/` for:

- validation/stat rune families
- graph/export ideas
- broader ergonomics
- self-host and future-language expansion
- Forge comparison notes that still inform future design

Use `veltra/` for:

- notebook format and UX
- product value and product roadmap
- Favnir vs Veltra boundary notes

## Current Root Policy

The root `dev/` directory should ideally contain only:

- this `README.md`
- category directories such as `v1/`, `post-v1/`, `veltra/`

New notes should go into one of those directories instead of being added directly to `dev/`.
