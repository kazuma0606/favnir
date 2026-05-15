# Favnir v1.4.0 Langspec

This version adds explainable artifacts and graph-oriented flow tooling.

## `fav explain --format json`

`fav explain` now supports machine-readable output.

```bash
fav explain examples/abstract_flw_basic.fav --format json
fav explain examples/abstract_flw_basic.fav --format json --focus trfs
```

The JSON payload includes:

- `schema_version`
- `favnir_version`
- `entry`
- `source`
- `fns`
- `trfs`
- `flws`
- `types`
- `effects_used`
- `emits`
- `runes_used`

Each function/transform/flow entry may include `reachable_from_entry`.

## Reachability

Reachability is computed from the executable entrypoint and used by:

- `fav explain --format json`
- `fav bundle`
- embedded explain metadata

Reported reachability includes:

- `included`
- `excluded`
- `effects_required`
- `emits`

## `fav bundle`

`fav bundle` produces a reachability-pruned artifact.

```bash
fav bundle examples/bundle_demo.fav -o dist/app.fvc --manifest --explain
```

Behavior:

- only entry-reachable functions/globals are kept
- `--manifest` writes a JSON reachability manifest
- `--explain` writes a sibling explain JSON file
- `--explain` also embeds explain JSON into the `.fvc` artifact

## Embedded explain metadata

When a bundled artifact contains embedded explain metadata, it can be read back directly:

```bash
fav explain dist/app.fvc --format json
```

Text mode on `.fvc` still falls back to artifact info.

## `fav graph`

`fav graph` renders flow structure in either text or Mermaid form.

```bash
fav graph examples/abstract_flw_basic.fav
fav graph examples/abstract_flw_basic.fav --format mermaid
```

Current graph output focuses on flow structure:

- `flw`
- `abstract flw`
- `flw binding`

## First-class trf type expressions

Function parameters may now use transform-shaped types directly:

```favnir
fn run(save: String -> Int !Db) -> Unit {
    ()
}
```

These parse as `TypeExpr::TrfFn` and are resolved as `Type::Trf`.

## `SlotImpl`

Bound abstract-flow slot implementations now carry structured binding information:

```rust
pub enum SlotImpl {
    Global(String),
    Local(String),
}
```

Parser-side syntax still starts as a named binding, and the checker resolves it semantically.

## Generic `abstract trf` shorthand in slots

Abstract flow slots may use generic abstract-transform shorthand:

```favnir
abstract trf Fetch<T>: Int -> T? !Db

abstract flw Pipeline<Row> {
    fetch: Fetch<Row>
}
```

Checker behavior:

- generic type arguments are substituted into the abstract-transform signature
- slot binding type mismatches still produce `E048`

## Error and warning codes

- `E048`: slot signature mismatch
- `E049`: unknown slot name in abstract flow binding
- `E050`: partial flow used in run/build
- `E051`: direct runtime call to `abstract trf`
- `W011`: partial flow reported in `fav check`

## Examples

- `examples/bundle_demo.fav`
- `examples/dynamic_inject.fav`
- `examples/abstract_flw_basic.fav`
- `examples/abstract_flw_inject.fav`
