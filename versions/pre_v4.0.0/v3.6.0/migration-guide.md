# Favnir v3.6.0 Migration Guide

## Overview

v3.6.0 adds incremental processing support. There are no breaking syntax changes from v3.5.0.

## New Features

### `!Checkpoint`

Declare `!Checkpoint` on functions that call `Checkpoint.*`.

```favnir
public fn main() -> Unit !Io !Checkpoint {
    bind last <- Checkpoint.last("etl_run")
    IO.println(Debug.show(last))
}
```

### `IO.timestamp()`

Use `IO.timestamp()` to produce a UTC timestamp string.

### `incremental` rune

```favnir
import rune "incremental"
```

Use `incremental.run_since` to wrap checkpoint lookup, fetch, and save.

### `fav.toml [checkpoint]`

```toml
[checkpoint]
backend = "file"
path = ".fav_checkpoints"
```

### `fav checkpoint`

Available commands:
- `fav checkpoint list`
- `fav checkpoint show <name>`
- `fav checkpoint reset <name>`
- `fav checkpoint set <name> <value>`

## No Manual Migration Required

Existing v3.5.0 code continues to run. Add the new effect and config only where incremental processing is needed.
