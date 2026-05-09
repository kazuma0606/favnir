# Favnir v2.0.0 Language Specification

## Breaking Changes from v1.x

### Removed keywords

The `trf`, `flw`, and `cap` keywords have been removed from the language.

| v1.x keyword | v2.0.0 replacement | Notes |
|---|---|---|
| `trf Name: A -> B = ...` | `stage Name: A -> B = ...` | Same semantics |
| `abstract trf Name: A -> B` | `abstract stage Name: A -> B` | Same semantics |
| `flw Name = A \|> B` | `seq Name = A \|> B` | Same semantics |
| `abstract flw Template<T> { ... }` | `abstract seq Template<T> { ... }` | Same semantics |
| `cap Name<T> = { method: T -> T }` | `interface Name { method: Self -> Self }` | Different syntax |

### Error codes for removed keywords

| Code | Trigger |
|---|---|
| E2001 | `trf` or `abstract trf` used in source code |
| E2002 | `flw` or `abstract flw` used in source code |
| E2003 | `cap` used in source code |

All three errors include a `fav migrate` hint.

---

## stage / seq (formerly trf / flw)

### stage definition

```favnir
stage Name: InputType -> OutputType !Effects = |param| {
    // body
}
```

A `stage` is a single-step pipeline transformer. It takes one input and produces one output.

### abstract stage

```favnir
abstract stage Name: InputType -> OutputType !Effects
```

An `abstract stage` declares a stage slot without an implementation. Used inside `abstract seq` templates.

### seq definition

```favnir
seq Name = StageA |> StageB |> StageC
```

A `seq` composes stages in order: the output of each stage becomes the input of the next.

### abstract seq

```favnir
abstract seq Template<T> {
    slot_a: InputType -> MiddleType
    slot_b: MiddleType -> OutputType !Effects
}
```

An `abstract seq` declares a pipeline template with named slots. Concrete sequences bind slots to stages:

```favnir
seq Concrete = Template<MyType> {
    slot_a <- MyStageA
    slot_b <- MyStageB
}
```

---

## fav migrate

`fav migrate` rewrites v1.x source files to v2.0.0 syntax automatically.

### Usage

```sh
fav migrate file.fav           # dry-run: show what would change
fav migrate --in-place file.fav  # rewrite file in place
fav migrate --check file.fav     # exit 1 if migration needed (CI)
fav migrate --dir src/           # migrate all .fav files in directory
```

### Transformations applied

| Pattern | Replacement |
|---|---|
| `trf Name` | `stage Name` |
| `flw Name` | `seq Name` |
| `abstract trf Name` | `abstract stage Name` |
| `abstract flw Name` | `abstract seq Name` |

Note: `cap` → `interface` is NOT auto-migrated due to syntax differences. Manual conversion is required.

---

## Selfhost Lexer Milestone

As of v2.0.0, the Favnir lexer for arithmetic operators is implemented in Favnir itself.
See `examples/selfhost/lexer.fav` for the implementation.

The selfhost lexer demonstrates:
- `String.length`, `String.char_at` for character-by-character processing
- `List.range` + `List.map` for indexed iteration
- `Option.unwrap_or` for safe character extraction
- `List.concat` for appending the `Eof` sentinel token

---

## Compatibility

v2.0.0 is a breaking release. All v1.x code using `trf`/`flw`/`cap` must be migrated.

`.fvc` artifacts built by v1.x (VERSION byte `0x06`) are not compatible with the v2.0.0 VM (VERSION byte `0x20`).

Run `fav migrate --in-place` to update source files automatically.
