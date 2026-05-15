# Favnir v3.1.0 Language Specification

## Overview

Favnir v3.1.0 keeps the v3.0.0 language surface intact and extends the toolchain with a local
documentation server via `fav docs`.

## Language

The core syntax, type system, effects, stages, seqs, imports, async functions, and explain JSON
schema remain the same as v3.0.0.

There are no source-level breaking changes in v3.1.0.

## Tooling Additions

### `fav docs`

```text
fav docs [file] [--port N] [--no-open]
```

Behavior:

- Starts a local HTTP server on `http://localhost:<port>`; default port is `7777`.
- Serves embedded UI assets at `/`, `/static/app.js`, and `/static/style.css`.
- Serves explain JSON at `/api/explain`.
- Serves embedded stdlib metadata at `/api/stdlib`.
- Opens the default browser unless `--no-open` is specified.
- If `file` is omitted, the project pane is empty and stdlib docs are still available.

### HTTP Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/` | Docs UI shell |
| `GET` | `/api/explain` | Explain JSON v3.0 |
| `GET` | `/api/stdlib` | Stdlib catalog JSON v3.1 |
| `GET` | `/static/app.js` | Embedded frontend logic |
| `GET` | `/static/style.css` | Embedded stylesheet |

### Stdlib Catalog

The docs server exposes these stdlib groups:

- `IO`: `println`, `print`, `read_line`
- `List`: `map`, `filter`, `fold`, `first`, `last`, `length`, `concat`, `range`, `take`, `drop`, `zip`, `join`, `sort`, `find`, `any`, `all`
- `Option`: `unwrap_or`, `map`, `is_some`, `is_none`
- `String`: `length`, `concat`, `slice`, `char_at`, `contains`, `split`, `trim`, `starts_with`, `ends_with`
- `Stream`: `of`, `from`, `gen`, `map`, `filter`, `take`, `to_list`
- `Debug`: `show`

## Explain JSON

`/api/explain` reuses the existing explain schema:

```json
{
  "schema_version": "3.0",
  "favnir_version": "3.1.0",
  "fns": [],
  "stages": [],
  "seqs": [],
  "types": []
}
```

When no file is provided, the docs server returns an empty explain payload with the same top-level
shape.

## Version Summary

- Language syntax: unchanged from v3.0.0
- CLI additions: `fav docs`, `--version`
- New embedded assets: `index.html`, `app.js`, `style.css`
