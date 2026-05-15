# Favnir v3.1.0 Implementation Plan

## Overview

v3.1.0 adds `fav docs` — a local documentation browser server.
Theme: minimal Rust HTTP server + embedded HTML/JS UI fed by explain JSON v3.0.

Total estimated phases: 7

---

## Phase 0: Version Bump

**Goal**: Update version strings to `3.1.0`.

- `fav/Cargo.toml`: `version = "3.1.0"`
- Run `cargo build` to propagate `env!("CARGO_PKG_VERSION")` everywhere.
- Verify `fav --version` prints `3.1.0`.

---

## Phase 1: HTTP Server Infrastructure

**Goal**: A minimal thread-per-connection HTTP/1.1 server in `src/docs_server.rs`.

Files:
- `src/docs_server.rs` (new)

Key items:
- `pub struct DocsServer { port: u16 }`
- `impl DocsServer { pub fn start(self, explain_json: String) -> ! }`
- `TcpListener::bind(format!("127.0.0.1:{}", self.port))`
- For each connection: read request line + headers, dispatch to handler, write HTTP/1.1 response.
- Route table:
  - `GET /` → serve embedded `index.html`
  - `GET /api/explain` → `explain_json` (passed in at start)
  - `GET /api/stdlib` → `build_stdlib_json()` (const table)
  - `GET /static/app.js` → embedded `app.js`
  - `GET /static/style.css` → embedded `style.css`
  - Other → 404
- Helper: `fn write_response(stream, status, content_type, body)`
- Shutdown: Ctrl-C handler prints `\nDocs server stopped.` + `std::process::exit(0)`.

Tests (in `docs_server.rs`):
- `docs_server_responds_to_root` — bind random port, send GET /, assert 200 + `<!DOCTYPE html>`.
- `docs_server_responds_to_api_explain` — assert JSON body contains `schema_version`.
- `docs_server_responds_to_api_stdlib` — assert JSON body contains `"modules"`.
- `docs_server_404_on_unknown` — assert 404.

---

## Phase 2: Stdlib JSON Catalog

**Goal**: Build the embedded stdlib JSON served at `/api/stdlib`.

Files:
- `src/docs_server.rs`: add `fn build_stdlib_json() -> String`

Key items:
- Define a const table `STDLIB_CATALOG: &[(&str, &[(&str, &str)])]` (module, [(fn_name, signature)]).
- Populate with: IO (println, print, read_line), List (all 12+), Option (3), String (9+), Stream (7), Debug (show).
- `build_stdlib_json()` serializes the table to the `/api/stdlib` JSON format (no serde dependency — hand-rolled).

Tests:
- `stdlib_json_contains_list_map` — assert output contains `"List.map"` or `"map"` under `"List"`.
- `stdlib_json_is_valid_json` — basic brace-balance check.

---

## Phase 3: Embedded UI Assets

**Goal**: Create the HTML/JS/CSS assets and embed them in the binary.

Files:
- `src/docs_assets/index.html` (new)
- `src/docs_assets/app.js` (new)
- `src/docs_assets/style.css` (new)
- `src/docs_server.rs`: add `include_str!` constants

### `index.html`
- Minimal HTML5 shell: `<input id="search">`, `<div id="left-pane">`, `<div id="right-pane">`.
- Loads `/static/app.js` and `/static/style.css`.

### `app.js`
- On load: `fetch('/api/explain')` + `fetch('/api/stdlib')` in parallel.
- Build left pane: Stdlib section (grouped by module), Project section (if explain has data).
- Click handler: render item detail in right pane.
- Search handler: filter left pane items, re-render.
- `renderDetail(item)`: show name, signature, params table, returns, effects.
- All vanilla JS — no framework, no bundler.

### `style.css`
- Two-column flex layout.
- Left pane: fixed 280px, scrollable.
- Right pane: flex-grow, scrollable.
- Monospace font for signatures (`font-family: monospace`).
- Light theme (white background, dark text).
- Active item highlight in left pane.

---

## Phase 4: `fav explain` JSON Generation (Internal)

**Goal**: Wire up explain JSON generation for the docs command.

Files:
- `src/driver.rs`: add `pub fn get_explain_json(file: &str) -> Result<String, String>`

Key items:
- Reuses the existing `cmd_explain` logic but returns JSON string instead of printing.
- Returns `Err(msg)` if file fails to parse/check.
- When `file` arg is `None` for `fav docs`, returns empty explain JSON stub.

Tests:
- `get_explain_json_returns_schema_version` — simple `.fav` file, assert output contains `"schema_version":"3.0"`.

---

## Phase 5: `cmd_docs` + CLI Integration

**Goal**: Wire up the `fav docs` command end-to-end.

Files:
- `src/driver.rs`: add `pub fn cmd_docs(file: Option<&str>, port: u16, no_open: bool)`
- `src/main.rs`: add `docs` subcommand parsing

### `cmd_docs`
```rust
pub fn cmd_docs(file: Option<&str>, port: u16, no_open: bool) {
    let explain_json = match file {
        Some(f) => get_explain_json(f).unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); }),
        None => empty_explain_json(),
    };
    println!("Favnir docs server running at http://localhost:{}", port);
    if !no_open {
        let _ = open::that(format!("http://localhost:{}", port));
    }
    DocsServer::new(port).start(explain_json);
}
```

### CLI parsing in `main.rs`
```
fav docs [file] [--port N] [--no-open]
```
- Default port: 7777.
- Add to HELP text.

### `Cargo.toml` additions
```toml
[dependencies]
open = "5"
```

---

## Phase 6: Tests

**Goal**: Integration tests for `cmd_docs` and server behavior.

Files:
- `src/driver.rs` (`docs_tests` module)

Test cases:
- `docs_server_start_stop` — start server on random port in a thread, GET `/`, assert 200, kill thread.
- `docs_server_stdlib_endpoint` — GET `/api/stdlib`, assert `"modules"` key present.
- `docs_server_explain_endpoint_empty` — no file arg, GET `/api/explain`, assert `"fns":[]`.
- `docs_server_explain_endpoint_with_file` — pass a temp `.fav` file with one `fn`, assert name appears in JSON.
- `docs_cmd_explain_json_helper` — unit test for `get_explain_json`.
- All existing tests still pass.

---

## Phase 7: Documentation

**Goal**: Create version docs.

Files:
- `versions/v3.1.0/langspec.md` — updated language spec (add `fav docs` to toolchain commands table)
- `versions/v3.1.0/migration-guide.md` — v3.0→v3.1 migration (no breaking changes; just new command)
- `versions/v3.1.0/progress.md` — all phases tracked

---

## Dependency Graph

```
Phase 0 (version)
    └── Phase 1 (HTTP server)
            └── Phase 2 (stdlib JSON)
            └── Phase 3 (UI assets)
            └── Phase 4 (explain JSON)
                    └── Phase 5 (cmd_docs + CLI)
                            └── Phase 6 (tests)
                                    └── Phase 7 (docs)
```

Phases 2, 3, 4 can be developed in parallel after Phase 1 is done.
