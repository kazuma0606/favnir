# v5.3.0 Spec — `rune` Package Manager

## Overview

v5.3.0 implements the `rune` CLI — Favnir's package manager for installing, publishing, and managing runes from the remote Rune Registry.

The `fav` binary gains a `rune` subcommand, and when symlinked/copied as `rune`, operates in package manager mode automatically.

---

## Commands

### `rune install [name[@version]]...`

Install one or more runes into `./rune_modules/<name>/`.

```
rune install csv
rune install csv@0.2.0
fav rune install csv parquet
```

**Steps:**
1. Read `./rune.toml` if present (to check existing deps)
2. `GET /runes/<name>` → get latest version (or use explicit version)
3. `GET /runes/<name>/download?version=<ver>` → base64-encoded zip
4. Decode base64 → unzip to `./rune_modules/<name>/`
5. Update `[dependencies]` in `./rune.toml` (create if absent)

**Output:**
```
Installing csv@0.2.0 ... done
Installing parquet@0.1.0 ... done
Updated rune.toml
```

---

### `rune uninstall <name>...`

Remove rune(s) from `./rune_modules/` and `./rune.toml` dependencies.

```
rune uninstall csv
```

**Steps:**
1. Delete `./rune_modules/<name>/` directory
2. Remove entry from `[dependencies]` in `./rune.toml`

---

### `rune list`

List installed runes (from `./rune_modules/` + `./rune.toml`).

```
rune list
```

**Output:**
```
csv       0.2.0   CSV parsing and serialization
parquet   0.1.0   Parquet file I/O
```

---

### `rune info <name>`

Show detailed info about a rune from the registry.

```
rune info csv
```

Calls `GET /runes/<name>` and displays name, version, description.

**Output:**
```
name:        csv
version:     0.2.0
description: CSV parsing and serialization
```

---

### `rune search <query>`

Search runes in the registry by name prefix or substring.

```
rune search par
```

Calls `GET /runes` and filters client-side by query string.

**Output:**
```
parquet   0.1.0   Parquet file I/O
```

---

### `rune update [name]`

Update installed rune(s) to latest version.

```
rune update          # update all in rune.toml
rune update csv      # update one
```

**Steps:**
1. Read `./rune.toml` to get current deps
2. For each (or specified) dep: fetch latest version from registry
3. If newer: run install flow, update `rune.toml`

---

### `rune publish`

Publish current rune to the registry. Requires `./rune.toml` with `[rune]` section.

```
rune publish
```

**Steps:**
1. Read `./rune.toml` → name, version, description, entry, effects
2. Collect all `.fav` files from the rune's directory
3. Create zip archive (in-memory)
4. Base64-encode
5. `POST /runes/<name>` with `{"version":..., "description":..., "zip":...}`
   - Auth: `Authorization: Basic <admin:adminuser base64>`
6. Print result

**Output:**
```
Publishing csv@0.2.0 ... done
```

---

## `rune.toml` Format (project-level)

When in a project directory (not inside a rune source dir), `rune.toml` tracks dependencies:

```toml
[dependencies]
csv = "0.2.0"
parquet = "0.1.0"
```

When inside a rune source directory, `rune.toml` has the `[rune]` section (existing format from v5.2.0):

```toml
[rune]
name = "csv"
version = "0.2.0"
description = "CSV parsing and serialization"
entry = "main.fav"
effects = ["Io", "Csv"]

[dependencies]
```

---

## Install Target

Runes are installed to `./rune_modules/<name>/`. The `.fav` compiler resolves `import csv` from `./rune_modules/csv/`.

---

## Remote Registry

Base URL: `https://32qp3qwhdh.execute-api.ap-northeast-1.amazonaws.com`

| Endpoint | Method | Description |
|---|---|---|
| `/runes` | GET | List all runes |
| `/runes/<name>` | GET | Get rune metadata |
| `/runes/<name>/versions` | GET | List all versions |
| `/runes/<name>/download?version=<v>` | GET | Download zip |
| `/runes/<name>` | POST | Publish rune |

---

## Binary Alias

When `argv[0]` ends with `rune` (the binary was invoked as `rune`), or when `fav rune ...` is used, the same code path executes.

To set up alias:
```bash
cp ~/.fav/bin/fav ~/.fav/bin/rune   # or symlink
```
