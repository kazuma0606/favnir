# Favnir v1.0.0 Language Specification

Date: 2026-05-01

This document is the consolidated language reference for Favnir v1.0.0.
It is intended to stabilize the language surface that was introduced across
v0.1.0 through v0.9.0 and refined for the v1.0.0 release.

## 1. Core Types

| Type | Description | Example |
|---|---|---|
| `Int` | 64-bit signed integer | `42`, `-7` |
| `Float` | 64-bit IEEE 754 float | `3.14` |
| `Bool` | Boolean | `true`, `false` |
| `String` | UTF-8 string | `"hello"` |
| `Unit` | Empty / no value | `()` |
| `List<T>` | Homogeneous list | `[1, 2, 3]` |
| `Map<V>` | String-keyed map | `Map.from_list(...)` |
| `Option<T>` | Nullable value (`some`/`none`) | `Option.some(x)` |
| `Result<T>` | Fallible value (`ok`/`err`) | `Result.ok(x)` |
| `T?` | Shorthand for `Option<T>` | `Int?` |
| `T!` | Shorthand for `Result<T>` | `String!` |
| `Pair<A,B>` | Two-element product | `{ first: a  second: b }` |

Records are declared with `type Name { field: Type ... }` (fields separated by spaces).
ADTs are declared with `type Name { Variant(Type) ... }`.

## 2. Functions, `trf`, and `flw`

- `fn`
- `trf`
- `flw`
- parameters and return types
- effects on callable definitions

## 3. Effect System

Effects are declared on functions with `!Effect` syntax. Multiple effects are written as `!Io !Db`.

| Effect | Trigger | Permission check |
|---|---|---|
| `Pure` | (none — default) | — |
| `Io` | `IO.println`, `IO.print` | required for any IO call |
| `Db` | `Db.*` builtins | E007 if missing |
| `Network` | `Http.*` builtins | E008 if missing |
| `File` | `File.*` builtins | E036 if missing |
| `Trace` | `Trace.print`, `Trace.log` | (always allowed; goes to stderr) |
| `Emit<T>` | `emit expr` | E009 if missing |

```favnir
public fn greet(name: String) -> Unit !Io {
    IO.println(name)
}
```

## 4. Pattern Matching

- literals
- variants
- records
- guards
- wildcard patterns

## 5. Modules and Runes

- `namespace`
- `use`
- visibility
- rune boundaries
- workspace boundaries

## 6. Standard Library Surface

- IO
- collections
- string
- JSON / CSV / file
- diagnostics-oriented builtins

## 7. CLI Reference

- `run`
- `check`
- `build`
- `exec`
- `test`
- `fmt`
- `lint`
- `explain`
- `lsp`

## 8. Error Codes

| Code | Description |
|---|---|
| E001 | Type mismatch |
| E002 | Undefined identifier |
| E003 | Pipeline / flw connection error |
| E004 | Effect violation |
| E005 | Arity mismatch |
| E006 | Pattern match error |
| E007 | Db effect missing |
| E008 | Network effect missing |
| E009 | Emit effect missing |
| E012 | Circular import |
| E013 | Module or symbol not found |
| E014 | Symbol not public (visibility violation) |
| E015 | Private symbol referenced from another file |
| E016 | Internal symbol referenced from outside rune |
| E018 | Unification failure (generic type mismatch) |
| E019 | Infinite type (occurs check failed) |
| E020 | Undefined capability |
| E021 | No implementation found for capability |
| E022 | Method not in capability |
| E023 | Arity mismatch (generic) |
| E024 | Chain statement outside Result/Option context |
| E025 | Chain type mismatch |
| E026 | Yield outside collect |
| E027 | Guard expression is not Bool |
| E032–E035 | Artifact read/write errors |
| E036 | File effect missing |
| W001 | WASM: unsupported type |
| W002 | WASM: unsupported expression |
| W003 | WASM: main signature must be `() -> Unit !Io` |
| W004 | `--db` cannot be used with `.wasm` artifacts |

## 9. Backward Compatibility Policy

Starting with v1.0.0, the Favnir language surface is considered stable:

- **Language syntax** (keywords, operators, statement forms) will not change in a breaking way within v1.x.
- **Error codes** E001–E040 and W001–W004 are stable; new codes are always additive.
- **CLI commands** (`run`, `build`, `exec`, `check`, `fmt`, `lint`, `test`, `explain`, `lsp`, `install`, `publish`) maintain their flag signatures within v1.x.
- **`.fvc` artifact format** version `0x06` is forward-readable by v1.x `exec`.
- **WASM codegen**: currently supports `Int`, `Float`, `Bool`, `String`, `Unit` scalars and closures that capture `Int`/`Float`/`Bool` values. `List<T>` and `Map<V>` in WASM are planned for v1.1.0.

Breaking changes (if any) will be gated behind a major version bump.

## 9. Example Programs

- single-file hello world
- pipeline-oriented examples
- test-oriented examples
- WASM-oriented examples

## Notes

- This file is intentionally a v1.0.0 scaffold first.
- Content should be consolidated from the existing `versions/v0.x.0/` specs
  without expanding scope beyond the v1.0.0 release surface.
