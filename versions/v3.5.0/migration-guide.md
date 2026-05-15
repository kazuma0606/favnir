# Favnir v3.5.0 Migration Guide

## Overview

v3.5.0 adds the `gen` rune for type-driven synthetic data generation.
There are **no breaking changes** from v3.4.0.

---

## New Features

### `Random.seed(n)` — deterministic RNG

```favnir
// Before (non-deterministic)
bind x <- Random.int(1, 100)

// After (deterministic, reproducible)
Random.seed(42);
bind x <- Random.int(1, 100)
```

Call `Random.seed(n)` once before any `Random.*` or `Gen.*` usage to fix
the RNG state for reproducible runs.

### `Gen.*` VM primitives

New built-in namespace `Gen` with five primitives:
- `Gen.string_val(len)` — random alphanumeric string
- `Gen.one_raw(type_name)` — one synthetic row as `Map<String, String>`
- `Gen.list_raw(type_name, n)` — N synthetic rows
- `Gen.simulate_raw(type_name, n, noise)` — rows with intentional corruption
- `Gen.profile_raw(type_name, data)` — data quality measurement → `GenProfile`

### `GenProfile` built-in type

Pre-registered type with fields `total`, `valid`, `invalid`, `rate`.
Available in all Favnir programs without declaration.

### `gen` rune

```favnir
import rune "gen"

type Sensor = { device_id: Int reading: Float unit: String }

public fn main() -> Unit !Io !Random {
    // Generate 50 clean rows
    bind clean <- gen.list("Sensor", 50, 42)

    // Generate 50 rows with 20% noise
    bind dirty <- gen.simulate("Sensor", 50, 0.2, 42)

    // Profile data quality
    bind report <- gen.profile("Sensor", dirty)
    IO.println($"Clean rate: {report.rate}")
}
```

### `fav check --sample N`

New flag on the `check` subcommand:

```bash
fav check pipeline.fav --sample 100
```

Generates synthetic data for the first record type in the file and
runs the pipeline, reporting any runtime errors.

---

## No Migration Required

All v3.4.0 code runs unchanged in v3.5.0.
