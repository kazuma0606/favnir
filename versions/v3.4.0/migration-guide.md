# Migration Guide: v3.3.0 → v3.4.0

## Summary

v3.4.0 adds the `fav infer` command. There are **no breaking changes**.
All v3.3.0 code compiles and runs without modification.

## New Additions

### `fav infer`

A new CLI command that infers Favnir type definitions from:
- CSV files (`fav infer data.csv`)
- SQLite DB tables (`fav infer --db sqlite:app.sqlite [table]`)
- PostgreSQL tables (`fav infer --db postgres://... [table]`, requires `postgres_integration` feature)

No migration needed — existing projects that do not use `fav infer` are unaffected.

### No new language primitives

`fav infer` is a code generation tool only. It does not add new runtime primitives,
effects, types, or VM opcodes. Existing `.fav` source files are 100% compatible.

## No-op Migration

No `fav migrate` command is needed for v3.4.0. Upgrade by updating your `fav` binary.
