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

## 5a. Interface System (v1.1.0)

`interface` キーワードで型の操作契約を宣言する。旧 `cap` の後継。

### 宣言

```fav
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {          -- Eq を前提とする
    compare: Self -> Self -> Int
}
```

`Self` は実装対象の型を指す特殊キーワード。
`interface Ord : Eq` は「Ord を実装するには Eq も必要」を意味する。

### 手書き実装（`impl`）

```fav
impl Show for Int {
    show = |x| Int.to_string(x)
}

impl Eq for UserRow {
    eq = |a b| a.id == b.id
}
```

### 自動合成（body-less `impl`）

```fav
-- UserRow の全フィールドが Show / Eq を持つ場合のみ有効
impl Show, Eq for UserRow
```

### `with` 糖衣構文

```fav
type UserRow with Show, Eq = { name: String  age: Int }
```

上記は `type UserRow = { ... }` + `impl Show, Eq for UserRow` と等価。

### 明示的な値渡し（暗黙解決なし）

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> { ... }

bind sorted <- sort(users, User.ord)
--                         ^^^^^^^^ Ord<User> の実装値を明示
```

### 組み込み interface

| interface | 対応型 | メソッド |
|---|---|---|
| `Show` | Int / Float / Bool / String | `show: Self -> String` |
| `Eq` | Int / Float / Bool / String | `eq: Self -> Self -> Bool` |
| `Ord` | Int / Float / String | `compare: Self -> Self -> Int` |
| `Gen` | Int / Float / Bool / String | `gen: Int? -> Self` |
| `Semigroup` | Int / Float | `combine: Self -> Self -> Self` |
| `Monoid` | Int / Float | `empty: Self` |
| `Group` | Int / Float | `inverse: Self -> Self` |
| `Ring` | Int / Float | `multiply: Self -> Self -> Self` |
| `Field` | Float | `divide: Self -> Self -> Self!` |

### `cap` との後方互換

旧 `cap` キーワードは v1.1.0 でも動作するが、`fav check` が W010 警告を出す。
`fav check --no-warn` で W010 を抑制できる。

```
warning[W010]: `cap` is deprecated. Use `interface` instead.
  --> src/main.fav:1:1
  |
1 | cap Show { ... }
  | ^^^ deprecated keyword
```

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
| E041 | 未定義の interface を `impl` しようとした |
| E042 | `impl` のメソッド型が interface シグネチャと不一致 |
| E043 | 値渡し時または `impl` 時に要求 interface が未実装 |
| E044 | 自動合成時にフィールドが interface を未実装 |
| W001 | WASM: unsupported type |
| W002 | WASM: unsupported expression |
| W003 | WASM: main signature must be `() -> Unit !Io` |
| W004 | `--db` cannot be used with `.wasm` artifacts |
| W010 | `cap` キーワードの使用（deprecated、v1.1.0〜） |

## 9. Backward Compatibility Policy

Starting with v1.0.0, the Favnir language surface is considered stable:

- **Language syntax** (keywords, operators, statement forms) will not change in a breaking way within v1.x.
- **Error codes** E001–E040 and W001–W004 are stable; new codes are always additive.
- **CLI commands** (`run`, `build`, `exec`, `check`, `fmt`, `lint`, `test`, `explain`, `lsp`, `install`, `publish`) maintain their flag signatures within v1.x.
- **`.fvc` artifact format** version `0x06` is forward-readable by v1.x `exec`.
- **WASM codegen**: currently supports `Int`, `Float`, `Bool`, `String`, `Unit` scalars and closures that capture `Int`/`Float`/`Bool` values. `List<T>` and `Map<V>` in WASM are future work.

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
