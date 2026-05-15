# Favnir v3.1.0 Specification

## Theme: `fav docs` — Local Reference UI

v3.1.0 adds a local documentation server that renders API reference in the browser,
styled after Swagger UI. The data source is the existing `fav explain --format json`
output (schema v3.0), so no new AST analysis is needed.

---

## 1. New Command: `fav docs`

### Syntax

```
fav docs [file] [--port N] [--no-open]
```

| Argument | Default | Description |
|----------|---------|-------------|
| `file` | _(none)_ | Entry `.fav` file; if omitted, shows stdlib only |
| `--port N` | `7777` | Port for HTTP server |
| `--no-open` | _(flag)_ | Skip auto-opening browser tab |

### Behavior

1. If `file` is provided, run `explain --format json` on it (internally) to get explain JSON.
2. Start an HTTP server on `localhost:<port>`.
3. Serve the embedded HTML/JS/CSS UI at `/`.
4. Serve the explain JSON data at `/api/explain`.
5. Serve stdlib metadata at `/api/stdlib`.
6. Unless `--no-open` is set, open `http://localhost:<port>` in the default browser.
7. Print `Favnir docs server running at http://localhost:<port>` to stdout.
8. Run until Ctrl-C.

---

## 2. HTTP Endpoints

| Method | Path | Response |
|--------|------|----------|
| `GET /` | — | Embedded HTML (UI shell) |
| `GET /api/explain` | — | explain JSON v3.0 (from file arg, or `{}` if no file) |
| `GET /api/stdlib` | — | JSON: stdlib module/function catalog |
| `GET /static/app.js` | — | Embedded JS (UI logic) |
| `GET /static/style.css` | — | Embedded CSS |

All responses include `Content-Type` headers. No external network calls are made.

---

## 3. UI Layout

```
+------------------+--------------------------------------+
|  [Search box]    |                                      |
+------------------+  RIGHT PANE: selected item detail    |
|  LEFT PANE       |                                      |
|  > Stdlib        |  fn add(x: Int, y: Int) -> Int       |
|    IO            |                                      |
|    List          |  Parameters:                         |
|    Option        |    x : Int                           |
|    String        |    y : Int                           |
|    Stream        |  Returns: Int                        |
|  > Project       |                                      |
|    stages        |  [source location if available]      |
|    seqs          |                                      |
|    fns           |                                      |
+------------------+--------------------------------------+
```

### Left Pane
- **Stdlib** section: collapsible groups per module (IO, List, Option, String, Stream, Debug).
- **Project** section: shown only when a file is passed; groups are `stages`, `seqs`, `fns`, `types`.
- Clicking any item loads its detail in the right pane.

### Search
- Filters both stdlib and project items by name (case-insensitive substring match).
- Updates left pane in real time; right pane clears on new search.

### Right Pane
- Shows: full signature, parameter table, return type, effect annotations, docstring (if present).
- For stages: shows input/output types.
- For seqs: shows pipeline chain.

---

## 4. Stdlib JSON Format (`/api/stdlib`)

```json
{
  "schema_version": "3.1",
  "modules": [
    {
      "name": "IO",
      "functions": [
        {
          "name": "println",
          "signature": "String -> Unit !Io",
          "params": [{"name": "s", "type": "String"}],
          "returns": "Unit",
          "effects": ["Io"]
        }
      ]
    }
  ]
}
```

This catalog is embedded in the binary (generated at build time from a const table in `driver.rs`).

---

## 5. explain JSON Integration

The `/api/explain` response is the standard explain JSON v3.0 document:

```json
{
  "schema_version": "3.0",
  "favnir_version": "3.1.0",
  "file": "main.fav",
  "fns": [...],
  "stages": [...],
  "seqs": [...],
  "types": [...]
}
```

When no file is provided, `/api/explain` returns `{"schema_version":"3.0","fns":[],"stages":[],"seqs":[],"types":[]}`.

---

## 6. Implementation Notes

- HTTP server: pure Rust, no external web framework. Use `std::net::TcpListener`.
- Each request is handled in a new thread (simple thread-per-connection, no async runtime).
- Embedded assets (HTML, JS, CSS) are inlined via `include_str!` macros at compile time.
- Browser open: `open::that(url)` via the `open` crate (cross-platform), or fallback to printing URL only.
- `--no-open` skips the `open::that` call.
- Server shutdown: on Ctrl-C (`ctrlc` crate or `SIGINT` handling), print `\nDocs server stopped.` and exit.

---

## 7. New Dependencies

| Crate | Version | Use |
|-------|---------|-----|
| `open` | `5.*` | Cross-platform browser open |

No other new runtime dependencies. The HTTP server is hand-rolled using `std::net`.

---

## 8. Asset Files

Located under `src/docs_assets/`:

| File | Description |
|------|-------------|
| `index.html` | UI shell; references `/static/app.js` and `/static/style.css` |
| `app.js` | Fetch + render logic (vanilla JS, no framework) |
| `style.css` | Layout + typography (light theme, monospace for signatures) |

---

## 9. Completion Criteria

- `fav docs` (no file) starts server at `http://localhost:7777`; stdlib modules visible.
- `fav docs main.fav` starts server; project functions/stages/seqs appear in left pane.
- Search filters items in real time.
- Clicking an item shows full signature in right pane.
- `--port 8080` changes the port.
- `--no-open` suppresses browser launch.
- All existing 647+ tests continue to pass.
- New tests: `cmd_docs_*` in driver.rs (server start/stop, endpoint responses).

---

## 10. Non-Goals for v3.1.0

- No authentication or TLS.
- No editing of source files via the UI.
- No hot-reload on file save (planned for v3.2.0).
- No dark mode toggle (future).
- No pagination (all items rendered at once).
