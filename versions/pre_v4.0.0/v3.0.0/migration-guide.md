# Favnir v3.0.0 Migration Guide

## Migrating from v2.x to v3.0.0

v3.0.0 is **backward compatible** with v2.0.0 source code.
No syntax changes are required; all v2.x programs run unchanged.

### What's New in v3.0.0

| Feature | Description |
|---------|-------------|
| `fav explain-error <code>` | Show detailed help for E0xxx error codes |
| `fav explain compiler` | Show the 5-step compilation pipeline |
| JSON schema v3.0 | `schema_version: "3.0"`, `stages`/`seqs` keys (was `trfs`/`flws`) |
| Selfhost lexer | `selfhost/lexer/lexer.fav` — full Favnir lexer in Favnir |
| Selfhost parser | `selfhost/parser/parser.fav` — arithmetic expression parser in Favnir |

### explain JSON Schema Changes

If you consume `fav explain --format json` output programmatically:

| Old key (v2.x) | New key (v3.0.0) |
|----------------|-----------------|
| `"trfs"` | `"stages"` |
| `"flws"` | `"seqs"` |
| `"schema_version": "1.0"` | `"schema_version": "3.0"` |
| `"favnir_version": "1.5.0"` | `"favnir_version": "3.0.0"` |

The `--focus trfs` flag still works (backward compat alias for `--focus stages`).

---

## Migrating from v1.x to v2.0.0+

### Automated Migration

```bash
fav migrate --in-place          # rewrite all files
fav migrate --dry-run           # preview changes
fav migrate --check             # CI: exit 1 if migration needed
fav migrate --dir src           # migrate all .fav files under src/
```

### Keyword Changes

| Old (v1.x) | New (v2.0.0+) | Notes |
|-----------|--------------|-------|
| `trf Name: A -> B = \|x\| ...` | `stage Name: A -> B = \|x\| ...` | Direct rename |
| `abstract trf Name: A -> B` | `abstract stage Name: A -> B` | Direct rename |
| `flw Pipeline = A \|> B` | `seq Pipeline = A \|> B` | Direct rename |
| `cap Interface { ... }` | `interface Interface { ... }` | **Manual migration required** |

> **Important**: `fav migrate` handles `trf` → `stage` and `flw` → `seq` automatically.
> The `cap` → `interface` migration requires manual editing because the syntax changed.

### Interface Syntax Change

**Before (v1.x with `cap`):**
```favnir
cap Show<T> {
    fn show(x: T) -> String
}

impl Show<User> {
    fn show(x: User) -> String { x.name }
}
```

**After (v2.0.0+ with `interface`):**
```favnir
interface Show {
    fn show(self: Self) -> String
}

impl Show for User {
    show = |u| u.name
}
```

### Error Code Changes

v3.0.0 uses 4-digit error codes (E0xxx). Old 3-digit codes (Exxx) were internal only.

| Code | Title |
|------|-------|
| E0101 | unexpected token |
| E0102 | expected expression |
| E0213 | type mismatch |
| E0901 | deprecated keyword `trf` — use `stage` |
| E0902 | deprecated keyword `flw` — use `seq` |
| E0903 | deprecated keyword `cap` — use `interface` |

Run `fav explain-error --list` for all codes.
Run `fav explain-error E0213` for detailed help.

---

## Breaking Changes Summary

| Version | Breaking Change |
|---------|----------------|
| v2.0.0 | `trf`/`flw` removed from parser (E0901/E0902 on use) |
| v2.0.0 | `cap` removed from parser (E0903 on use) |
| v2.0.0 | `.fvc` artifact format version bumped (0x06 → 0x20); rebuild required |
| v3.0.0 | explain JSON: `trfs`/`flws` keys renamed to `stages`/`seqs` |

---

## Compatibility Policy

- Source compatibility: guaranteed within a major version (v3.x).
- `.fvc` artifacts: rebuild required when major version changes.
- explain JSON schema: `schema_version` field tracks breaking changes.
