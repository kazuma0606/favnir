# Favnir v3.6.0 Language Specification

## Overview

v3.6.0 adds incremental processing primitives:
- `!Checkpoint`
- `Checkpoint.last/save/reset/meta`
- `IO.timestamp()`
- `DB.upsert_raw(...)`
- `incremental` rune
- `fav.toml [checkpoint]`
- `fav checkpoint ...`

## Effects

`!Checkpoint` is a built-in effect. Functions that call `Checkpoint.*` must declare it.

```favnir
public fn main() -> Unit !Io !Checkpoint {
    bind last <- Checkpoint.last("etl_run")
    IO.println(Debug.show(last))
}
```

## Checkpoint API

```favnir
Checkpoint.last(name: String) -> Option<String> !Checkpoint
Checkpoint.save(name: String, value: String) -> Unit !Checkpoint
Checkpoint.reset(name: String) -> Unit !Checkpoint
Checkpoint.meta(name: String) -> CheckpointMeta !Checkpoint
```

`CheckpointMeta` is a built-in record type:

```favnir
type CheckpointMeta = {
    name: String
    value: String
    updated_at: String
}
```

## Timestamp

```favnir
IO.timestamp() -> String !Io
```

Returns an ISO 8601 UTC timestamp such as `"2026-05-15T12:34:56Z"`.

## DB Upsert

```favnir
DB.upsert_raw(conn: DbHandle, type_name: String, row: Map<String, String>, key_field: String) -> Unit !Db
```

This performs an idempotent insert/update keyed by `key_field`.

## Incremental Rune

```favnir
import rune "incremental"
```

Public API:
- `incremental.last`
- `incremental.save`
- `incremental.reset`
- `incremental.meta`
- `incremental.run_since`
- `incremental.upsert`

`run_since` accepts a closure:

```favnir
bind rows <- incremental.run_since("etl_run", |since|
    DB.query_raw(conn, $"SELECT * FROM events WHERE ts > '{since}'")
)
```

It reads the previous checkpoint, calls the fetch function, saves `IO.timestamp()`, and returns the rows.

## fav.toml

```toml
[checkpoint]
backend = "file"
path = ".fav_checkpoints"
```

Supported backends:
- `file`
- `sqlite`

For `sqlite`, `path` is the database file path.
