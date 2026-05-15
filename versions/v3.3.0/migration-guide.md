# Migration Guide: v3.2.0 → v3.3.0

## Summary

v3.3.0 adds the `db` rune and related primitives. There are **no breaking changes**.
All v3.2.0 code compiles and runs without modification.

## New Additions

### `import rune "db"`

A new official rune for SQL database access. No migration needed — existing code that does not
import `db` is unaffected.

### `DbError`, `DbHandle`, `TxHandle` types

These are new stdlib types. They will only conflict with user-defined types if you have already
defined types named `DbError`, `DbHandle`, or `TxHandle`. In that case, rename your user-defined
types.

### `!Db` effect

Functions that call any `DB.*` primitive must declare `!Db` in their signature. This is a new
compile-time requirement. If you were using the old lowercase `Db.*` builtins, they already
required `!Db` and are unaffected.

### `Env.get` / `Env.get_or`

New environment variable primitives. `Env.get` returns `Result<String, DbError>`. `Env.get_or`
is pure and returns `String` directly.

### Lint L008

A new lint warning is emitted when `DB.connect` or `db.connect` is called with a string literal
containing credentials (e.g. `"postgres://user:pass@..."``). To suppress the warning, use
`Env.get_or("DB_URL", "sqlite::memory:")` instead.

## No-op Migration

No `fav migrate` command is needed for v3.3.0. Upgrade by updating your `fav` binary.
