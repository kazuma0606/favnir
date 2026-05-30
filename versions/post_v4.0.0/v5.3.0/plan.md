# v5.3.0 Plan — `rune` Package Manager Implementation

## Scope

All changes are confined to `fav/src/`. No VM/checker changes needed — this is pure CLI work using existing HTTP (`ureq`) and zip crates.

---

## File Changes

### 1. `fav/src/main.rs`

**Add `argv[0]` detection:**
```rust
let exe_name = std::env::args().next()
    .map(|s| std::path::Path::new(&s).file_stem()
        .and_then(|s| s.to_str()).unwrap_or("fav").to_string())
    .unwrap_or_default();

if exe_name == "rune" {
    // treat as: fav rune <args>
    let rune_args: Vec<String> = std::env::args().skip(1).collect();
    return cmd_rune(&rune_args);
}
```

**Add `rune` subcommand dispatch:**
```rust
Some("rune") => {
    let sub_args: Vec<String> = args.iter().skip(1).map(|s| s.to_string()).collect();
    cmd_rune(&sub_args)
}
```

**Add import:**
```rust
use driver::cmd_rune;
```

### 2. `fav/src/driver.rs` (or new `fav/src/rune_cmd.rs`)

New public function `cmd_rune(args: &[String])` that dispatches:

```rust
pub fn cmd_rune(args: &[String]) -> i32 {
    match args.first().map(|s| s.as_str()) {
        Some("install")   => cmd_rune_install(&args[1..]),
        Some("uninstall") => cmd_rune_uninstall(&args[1..]),
        Some("list")      => cmd_rune_list(),
        Some("info")      => cmd_rune_info(&args[1..]),
        Some("search")    => cmd_rune_search(&args[1..]),
        Some("update")    => cmd_rune_update(&args[1..]),
        Some("publish")   => cmd_rune_publish(),
        _ => { eprintln!("Usage: rune <install|uninstall|list|info|search|update|publish>"); 1 }
    }
}
```

### 3. New `fav/src/rune_cmd.rs`

Contains all `cmd_rune_*` implementations plus helpers:

#### Registry client helpers

```rust
const REGISTRY_URL: &str = "https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com";

fn registry_get(path: &str) -> Result<String, String>
fn registry_post(path: &str, body: &str, auth: &str) -> Result<(u16, String), String>
```

Use `ureq::get/post` (already a dependency). No SigV4 needed — API Gateway has no IAM auth on GET; publish uses Basic auth via `Authorization` header.

#### `cmd_rune_install(names: &[String]) -> i32`

```
for each name[@version]:
  parse name and optional version
  if no version: GET /runes/<name> → parse version field
  GET /runes/<name>/download?version=<ver> → JSON with "body" (base64 zip)
  base64_decode(body) → bytes
  unzip bytes → ./rune_modules/<name>/
  update_rune_toml_dep(name, version)
```

#### `cmd_rune_uninstall(names: &[String]) -> i32`

```
for each name:
  fs::remove_dir_all("./rune_modules/<name>")
  remove_rune_toml_dep(name)
```

#### `cmd_rune_list() -> i32`

Read `./rune_modules/` dirs. For each dir, try to read `./rune_modules/<name>/rune.toml` for version/description. Print table.

#### `cmd_rune_info(args: &[String]) -> i32`

GET `/runes/<name>` → print fields.

#### `cmd_rune_search(args: &[String]) -> i32`

GET `/runes` → parse array → filter by query → print table.

#### `cmd_rune_update(args: &[String]) -> i32`

Read `./rune.toml` deps. For each (or specified): fetch latest version. If different from installed, re-install.

#### `cmd_rune_publish() -> i32`

```
read ./rune.toml → [rune] section
collect .fav files from current dir (walkdir)
zip in-memory (zip crate)
base64 encode (base64 crate)
POST /runes/<name> with JSON body + Basic auth header
```

### 4. `fav/src/rune_toml.rs`

TOML read/write helpers for project-level `rune.toml`:

```rust
pub struct RuneToml {
    pub rune: Option<RuneMeta>,       // [rune] section (source dir)
    pub dependencies: BTreeMap<String, String>,  // [dependencies]
}

pub fn read_rune_toml(path: &Path) -> Result<RuneToml, String>
pub fn write_rune_toml(path: &Path, toml: &RuneToml) -> Result<(), String>
pub fn add_dep(path: &Path, name: &str, version: &str) -> Result<(), String>
pub fn remove_dep(path: &Path, name: &str) -> Result<(), String>
```

Use `serde_yaml` (already a dep) — actually use manual TOML parsing since `serde_yaml` handles YAML not TOML. Use `toml` crate or manual string manipulation.

**Decision**: Use `toml` crate (add to Cargo.toml) for clean serde-based parsing.

Alternatively, since `rune.toml` is simple, use manual `serde_yaml`-style or just hand-parse with string split. **Use `toml` crate** — cleanest.

### 5. `fav/Cargo.toml`

Add:
```toml
toml = { version = "0.8", features = ["preserve_order"] }
```

---

## Zip Operations

**Unzip** (install): `zip::ZipArchive` → iterate entries → write files to `./rune_modules/<name>/`

**Zip** (publish): `zip::ZipWriter` → write each `.fav` file → finalize → `.finish()` → `Vec<u8>`

Both use the existing `zip = "0.6"` dependency.

---

## Error Handling

All `cmd_rune_*` return `i32` (0 = success, 1 = error), printing to stderr on error. This matches existing CLI conventions in `main.rs`.

---

## Module Registration

In `fav/src/main.rs` or `fav/src/driver.rs`, add:
```rust
mod rune_cmd;
mod rune_toml;
```

---

## Test Plan

Add to `fav/src/backend/vm_stdlib_tests.rs` or a new `fav/src/rune_cmd_tests.rs`:
- `test_rune_toml_read_write` — round-trip rune.toml add/remove dep
- `test_zip_roundtrip` — zip files in-memory, unzip, verify contents
- Integration test for `cmd_rune_install` requires live registry → mark `#[ignore]`

---

## Implementation Order

1. `rune_toml.rs` — pure data layer, no network
2. `rune_cmd.rs` skeleton — compile-clean, all stubs returning 0
3. Implement `cmd_rune_publish` (easiest: local files → POST)
4. Implement `cmd_rune_install` (network + unzip)
5. Implement remaining commands (`list`, `info`, `search`, `uninstall`, `update`)
6. Wire into `main.rs` and `driver.rs`
7. Tests
