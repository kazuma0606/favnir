# Favnir v3.1.0 Tasks

## Phase 0: Version Bump

- [x] `fav/Cargo.toml`: set `version = "3.1.0"`
- [x] `cargo build` succeeds; `env!("CARGO_PKG_VERSION")` propagated
- [x] `fav --version` prints `favnir 3.1.0`

## Phase 1: HTTP Server Infrastructure

- [x] Create `src/docs_server.rs`
- [x] `pub struct DocsServer { port: u16, shutdown: Arc<AtomicBool> }` + `DocsServer::new(port: u16)`
- [x] `TcpListener::bind("127.0.0.1:{port}")` with `set_nonblocking(true)` + accept loop
- [x] Request parser: read request line from `TcpStream`
- [x] Route dispatch: `GET /`, `/api/explain`, `/api/stdlib`, `/static/app.js`, `/static/style.css`, 404 fallback
- [x] `fn write_response(stream, status_code, content_type, body)` helper
- [x] Ctrl-C handler: OS-specific (`SetConsoleCtrlHandler` on Windows, `signal(SIGINT)` on Unix); sets `CTRL_C_SHUTDOWN` atomic flag
- [x] `DocsServer::stop()` for test teardown (sets `shutdown` flag + wake-up connect)
- [x] Test: `docs_server_responds_to_root` (assert 200 + `<!DOCTYPE html>`)
- [x] Test: `docs_server_responds_to_api_explain`
- [x] Test: `docs_server_responds_to_api_stdlib`
- [x] Test: `docs_server_404_on_unknown`

## Phase 2: Stdlib JSON Catalog

- [x] Define `STDLIB_CATALOG: &[StdlibModule]` const table in `docs_server.rs`
- [x] `StdlibModule`, `StdlibFunction`, `StdlibParam` structs with `#[derive(Serialize, Clone, Copy)]`
- [x] Populate IO: `println`, `print`, `read_line`
- [x] Populate List: `map`, `filter`, `fold`, `first`, `last`, `length`, `concat`, `range`, `take`, `drop`, `zip`, `join`, `sort`, `find`, `any`, `all`
- [x] Populate Option: `unwrap_or`, `map`, `is_some`, `is_none`
- [x] Populate String: `length`, `concat`, `slice`, `char_at`, `contains`, `split`, `trim`, `starts_with`, `ends_with`
- [x] Populate Stream: `of`, `from`, `gen`, `map`, `filter`, `take`, `to_list`
- [x] Populate Debug: `show`
- [x] `pub fn build_stdlib_json() -> String` via `serde_json::to_string` (uses `Serialize` derive)
- [x] Test: `stdlib_json_contains_list_map`
- [x] Test: `stdlib_json_is_valid_json`

## Phase 3: Embedded UI Assets

- [x] Create `src/docs_assets/` directory
- [x] Create `src/docs_assets/index.html` (HTML5 shell: topbar search + two-pane layout)
- [x] Create `src/docs_assets/app.js` (fetch `/api/explain` + `/api/stdlib`; render left pane; click detail; search filter — vanilla JS)
- [x] Create `src/docs_assets/style.css` (two-column flex; monospace sigs; light theme)
- [x] `include_str!` constants `INDEX_HTML`, `APP_JS`, `STYLE_CSS` in `docs_server.rs`
- [x] Asset constants wired into route handlers

## Phase 4: Explain JSON Helper

- [x] Add `pub fn get_explain_json(file: &str) -> Result<String, String>` in `driver.rs`
- [x] Reuses existing `try_load_explain_json` pipeline; returns JSON string via `serde_json::to_string`
- [x] Add `fn empty_explain_json() -> String` (stub with empty arrays, `schema_version: "3.0"`)
- [x] Test: `get_explain_json_returns_schema_version`

## Phase 5: `cmd_docs` + CLI Integration

- [x] Add `open = "5"` to `fav/Cargo.toml` `[dependencies]`
- [x] Add `pub fn cmd_docs(file: Option<&str>, port: u16, no_open: bool)` in `driver.rs`
- [x] Call `get_explain_json` (or `empty_explain_json`) based on file arg
- [x] Print `Favnir docs server running at http://localhost:{port}`
- [x] Call `open::that(url)` unless `--no-open`
- [x] Call `DocsServer::new(port).start(explain_json)` (returns `Result`; exits on error)
- [x] Add `docs` subcommand to `main.rs` argument parser
- [x] Parse `--port N` (default 7777) and `--no-open` flag
- [x] Add `fav docs` entry to HELP text

## Phase 6: Integration Tests

- [x] Test helpers in `tests` module: `http_get(port, path)`, `start_docs_server_for_test(explain_json)`
  - Note: tests live in the existing `tests` module (not a separate `docs_tests` module)
- [x] Test: `docs_server_start_stop` (spawn thread, GET `/`, assert 200)
- [x] Test: `docs_server_stdlib_endpoint` (assert `"modules"` in response)
- [x] Test: `docs_server_explain_endpoint_empty` (assert `"fns":[]`)
- [x] Test: `docs_server_explain_endpoint_with_file` (temp file with one fn, assert `"name":"add"` in JSON)
- [x] All existing tests still pass
- [x] `cargo test` green

## Phase 7: Documentation

- [x] Create `versions/v3.1.0/langspec.md` (adds `fav docs` to toolchain; no language changes)
- [x] Create `versions/v3.1.0/migration-guide.md` (no breaking changes; new command only)
- [x] Update `versions/v3.1.0/progress.md` (all phases marked `[x]`)
