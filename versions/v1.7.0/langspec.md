# Favnir v1.7.0 Language Specification

## Task<T> Async Base

### Syntax

```favnir
async fn fetch() -> String !Io {
    "hello"
}

async trf Process: Int -> String !Io = |x| Int.show.show(x)
```

The `async` keyword may precede `fn` or `trf`. The declared return type `T` is
automatically wrapped: the function's actual type becomes `() -> Task<T>`.

### Type wrapping

| Declaration | Inferred type |
|---|---|
| `async fn f() -> Int` | `() -> Task<Int>` |
| `async fn g(x: String) -> Bool` | `String -> Task<Bool>` |

### bind unwrapping

`bind x <- expr` transparently unwraps `Task<T>` → `T` at type-check time:

```favnir
bind n <- fetch()   // n: Int, not Task<Int>
```

### Task builtins

| Builtin | Signature | Behaviour |
|---|---|---|
| `Task.run(v)` | `T -> T` | Returns `v` as-is (synchronous) |
| `Task.map(v, f)` | `T -> (T -> U) -> U` | Applies `f` to `v` |
| `Task.and_then(v, f)` | `T -> (T -> U) -> U` | Applies `f` to `v` (flat) |

At runtime, `Task<T>` is transparent — the value is stored directly with no
wrapper allocation. Full async execution is planned for v1.8.0 (tokio).

---

## Type Aliases

### Syntax

```favnir
type UserId   = Int
type UserName = String
type MaybeId  = Option<Int>
```

### Semantics

- A type alias is fully compatible with its target type.
- Aliases may reference built-in or user-defined types.
- Generic targets are supported: `type MaybeInt = Option<Int>`.
- Aliases are resolved at type-check time; no runtime overhead.

### Restrictions (current version)

- Generic alias parameters (`type Pair<A, B> = ...`) are not yet supported —
  planned for v1.8.0.
- Circular aliases are rejected at parse/check time.

---

## fav test --coverage

Track which source lines were executed during test runs:

```text
fav test --coverage
fav test --coverage src/main.fav
```

Output format:

```
coverage: src/main.fav
  lines covered: 12 / 15 (80.0%)
  uncovered:     lines 7, 9, 14
```

- Blank lines, comment lines, and brace-only lines are excluded from the
  denominator.
- Coverage data is reset between test runs.

---

## fav watch --dir / --debounce

Watch multiple directories and run a command on any change:

```text
fav watch --cmd check --dir src --dir tests
fav watch --cmd test  --debounce 500
```

| Flag | Default | Description |
|---|---|---|
| `--cmd` | `check` | Command to run: `check`, `test`, or `run` |
| `--dir` | (none) | Extra directories to watch (repeatable) |
| `--debounce` | `300` | Debounce interval in milliseconds |

---

## Error codes added in v1.7.0

| Code | Phase | Message |
|---|---|---|
| E057 | checker | `async` keyword on non-`fn`/`trf` item |
| E058 | vm | `Task.run` received a non-Task value |
| E059 | checker | type alias target is undefined |
| E060 | checker | circular type alias |
