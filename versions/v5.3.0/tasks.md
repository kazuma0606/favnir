# v5.3.0 Tasks — `rune` Package Manager

## Phase A — Foundation

- [x] A-1: Add `toml = "0.8"` to `fav/Cargo.toml`
- [x] A-2: Create `fav/src/rune_toml.rs` — `RuneToml` struct, `read_rune_toml`, `write_rune_toml`, `add_dep`, `remove_dep`
- [x] A-3: Unit tests for `rune_toml.rs` — round-trip read/write, add/remove dep
- [x] A-4: Create `fav/src/rune_cmd.rs` — skeleton with all `cmd_rune_*` stubs returning 0
- [x] A-5: Wire `mod rune_cmd; mod rune_toml;` into `fav/src/main.rs`
- [x] A-6: Add `Some("rune") => cmd_rune(...)` dispatch in `main.rs`
- [x] A-7: Add `argv[0] == "rune"` detection in `main.rs`
- [x] A-8: All 937+ existing tests still pass (`cargo test`)

## Phase B — Publish

- [x] B-1: Implement `cmd_rune_publish` — read `./rune.toml` `[rune]` section
- [x] B-2: Implement zip creation — collect `.fav` files with `walkdir`, zip in-memory with `zip::ZipWriter`
- [x] B-3: Implement `registry_post` — `ureq` POST with `Authorization: Basic` header
- [x] B-4: E2E test: `rune publish` from `rune-registry/` or a test rune dir → 201 Published
- [x] B-5: Tests pass

## Phase C — Install

- [x] C-1: Implement `registry_get` helper — `ureq::get` with JSON response parsing
- [x] C-2: Implement `cmd_rune_install` for single rune with explicit version
- [x] C-3: Implement version resolution — `GET /runes/<name>` when no version specified
- [x] C-4: Implement base64 decode + unzip to `./rune_modules/<name>/`
- [x] C-5: Implement `update_rune_toml_dep` call after successful install
- [x] C-6: Support multiple names in one invocation (`rune install csv parquet`)
- [x] C-7: E2E test: `rune install csv` → `./rune_modules/csv/` exists, `./rune.toml` updated
- [x] C-8: Tests pass

## Phase D — Remaining Commands

- [x] D-1: Implement `cmd_rune_uninstall` — `fs::remove_dir_all` + `remove_dep`
- [x] D-2: Implement `cmd_rune_list` — read `./rune_modules/` dirs + `rune.toml` metadata
- [x] D-3: Implement `cmd_rune_info` — `GET /runes/<name>` + formatted output
- [x] D-4: Implement `cmd_rune_search` — `GET /runes` + client-side filter
- [x] D-5: Implement `cmd_rune_update` — read deps, compare versions, reinstall if newer
- [x] D-6: E2E test: uninstall → list (empty), info, search, update
- [x] D-7: Tests pass (all 937+ tests)

## Phase E — Polish & Docs

- [x] E-1: Help text for all subcommands (`rune --help`, `rune install --help`)
- [x] E-2: Update `versions/v5.0.0/tasks.md` → mark v5.3.0 complete
- [x] E-3: Update `MEMORY.md` with v5.3.0 key patterns
- [x] E-4: Commit `feat: Implement rune package manager (v5.3.0)`
