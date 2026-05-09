# Favnir v2.0.0 Migration Guide

## Overview

v2.0.0 removes three deprecated keywords (`trf`, `flw`, `cap`) and replaces them with
their canonical alternatives (`stage`, `seq`, `interface`). This guide explains how to
migrate your codebase.

## Automated migration

For `trf` → `stage` and `flw` → `seq`, use `fav migrate`:

```sh
# Preview what will change
fav migrate --dry-run src/

# Apply changes in-place
fav migrate --in-place src/

# CI: fail if any file needs migration
fav migrate --check src/
```

## Manual migration: cap → interface

`cap` cannot be auto-migrated because the syntax is different.

### Before (v1.x)

```favnir
cap Eq<T> = {
    equals: T -> T -> Bool
}

impl Eq<Int> {
    fn equals(a: Int, b: Int) -> Bool { a == b }
}
```

### After (v2.0.0)

```favnir
interface Eq {
    equals: Self -> Self -> Bool
}

impl Eq for Int {
    equals = |a b| a == b
}
```

Key differences:
- `cap Eq<T>` → `interface Eq` (no explicit type parameter; `Self` refers to the implementing type)
- Method types use `Self` instead of `T`
- `impl Cap<Type>` → `impl Cap for Type`
- Method bodies: `fn name(params) -> Ret { body }` → `name = |params| body`

## Full keyword mapping

| v1.x | v2.0.0 | Auto-migrate? |
|---|---|---|
| `trf Name: A -> B = \|p\| body` | `stage Name: A -> B = \|p\| body` | ✅ Yes |
| `abstract trf Name: A -> B` | `abstract stage Name: A -> B` | ✅ Yes |
| `flw Name = A \|> B` | `seq Name = A \|> B` | ✅ Yes |
| `abstract flw Template<T> { ... }` | `abstract seq Template<T> { ... }` | ✅ Yes |
| `flw Bound = Template<T> { slot <- Impl }` | `seq Bound = Template<T> { slot <- Impl }` | ✅ Yes |
| `cap Name<T> = { method: T -> T }` | `interface Name { method: Self -> Self }` | ❌ Manual |

## Error codes

If you see these errors after upgrading to v2.0.0, run `fav migrate`:

- `E2001`: keyword `trf` has been removed; use `stage` instead
- `E2002`: keyword `flw` has been removed; use `seq` instead  
- `E2003`: keyword `cap` has been removed; use `interface` instead

## .fvc artifact compatibility

Artifacts built with v1.x (VERSION `0x06`) must be rebuilt with v2.0.0 (VERSION `0x20`).
Re-run `fav build` after migrating your source files.
