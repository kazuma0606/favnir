# Favnir v2.0.0 Release Notes

**Released: 2026-05-09**

## Breaking Changes

### Removed keywords: `trf`, `flw`, `cap`

The `trf`, `flw`, and `cap` keywords have been removed from the language.
These were deprecated aliases since v1.x in favour of `stage`, `seq`, and `interface`.

- `trf` → `stage` (auto-migrate: `fav migrate --in-place`)
- `flw` → `seq` (auto-migrate: `fav migrate --in-place`)
- `cap` → `interface` (manual migration required)

Using old keywords now produces parse errors with migration hints:
- E2001: keyword `trf` has been removed
- E2002: keyword `flw` has been removed
- E2003: keyword `cap` has been removed

### .fvc artifact VERSION bump

The FVC binary format version byte changed from `0x06` to `0x20`.
v1.x artifacts must be rebuilt with `fav build`.

## New Features

### `fav migrate` command

Automatically rewrites v1.x source files to v2.0.0 syntax.

```sh
fav migrate --dry-run src/          # preview changes
fav migrate --in-place src/         # apply changes
fav migrate --check src/            # CI: exit 1 if needed
```

### Selfhost Lexer Milestone

A Favnir lexer for arithmetic operators is now implemented in Favnir itself
(`examples/selfhost/lexer.fav`). This demonstrates Favnir's expressive power
for string processing using `String.char_at`, `List.range`, and `List.map`.

## Test Coverage

538 unit tests passing (up from 509 in v1.8.0).

## Migration

See `versions/v2.0.0/migration-guide.md` for a complete migration checklist.
