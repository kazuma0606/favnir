# Favnir v2.1.0 Language Specification

Date: 2026-05-13

This document records the language and CLI surface added in Favnir v2.1.0.
It is additive over v2.0.0.

## 1. Summary

Favnir v2.1.0 adds:

- Math builtins
- new List and String builtins
- `IO.read_line()`
- logical operators `&&` and `||`
- `fav new`
- a welcome screen when `fav` is run with no arguments

v2.1.0 does not reintroduce removed v1.x keywords. `trf`, `flw`, and `cap`
remain removed as defined in v2.0.0.

## 2. Standard Library Additions

### 2.1 Math

| Symbol | Type |
|---|---|
| `Math.abs` | `Int -> Int` |
| `Math.abs_float` | `Float -> Float` |
| `Math.min` | `Int -> Int -> Int` |
| `Math.max` | `Int -> Int -> Int` |
| `Math.min_float` | `Float -> Float -> Float` |
| `Math.max_float` | `Float -> Float -> Float` |
| `Math.clamp` | `Int -> Int -> Int -> Int` |
| `Math.pow` | `Int -> Int -> Int` |
| `Math.pow_float` | `Float -> Float -> Float` |
| `Math.sqrt` | `Float -> Float` |
| `Math.floor` | `Float -> Int` |
| `Math.ceil` | `Float -> Int` |
| `Math.round` | `Float -> Int` |
| `Math.pi` | `Float` |
| `Math.e` | `Float` |

Notes:

- `Math.pow` requires a non-negative exponent.
- `Math.pi` and `Math.e` are namespace fields, not functions.

### 2.2 List

| Symbol | Type |
|---|---|
| `List.unique` | `List<T> -> List<T>` |
| `List.flatten` | `List<List<T>> -> List<T>` |
| `List.chunk` | `List<T> -> Int -> List<List<T>>` |
| `List.sum` | `List<Int> -> Int` |
| `List.sum_float` | `List<Float> -> Float` |
| `List.min` | `List<Int> -> Option<Int>` |
| `List.max` | `List<Int> -> Option<Int>` |
| `List.count` | `List<T> -> (T -> Bool) -> Int` |

Notes:

- `List.unique` preserves first-seen order.
- `List.flatten` flattens one level only.
- `List.chunk` requires a positive chunk size.
- `List.sum([])` returns `0`.
- `List.sum_float([])` returns `0.0`.
- `List.min([])` and `List.max([])` return `None`.

### 2.3 String

| Symbol | Type |
|---|---|
| `String.index_of` | `String -> String -> Option<Int>` |
| `String.pad_left` | `String -> Int -> String -> String` |
| `String.pad_right` | `String -> Int -> String -> String` |
| `String.reverse` | `String -> String` |
| `String.lines` | `String -> List<String>` |
| `String.words` | `String -> List<String>` |

Notes:

- `String.index_of` returns a zero-based index.
- `String.pad_left` and `String.pad_right` require a non-empty fill string.
- `String.lines` follows Rust `str::lines()` behavior.
- `String.words` splits on Unicode whitespace.

### 2.4 IO

| Symbol | Type | Effects |
|---|---|---|
| `IO.read_line` | `() -> String` | `!Io` |

Notes:

- `IO.read_line()` reads one line from standard input.
- A trailing `\n` or `\r\n` is removed from the returned string.
- In suppressed test IO mode, `IO.read_line()` returns `""`.

## 3. Logical Operators

Favnir v2.1.0 adds infix logical operators:

- `a && b`
- `a || b`

### 3.1 Types

- `&&` requires both operands to be `Bool` and returns `Bool`.
- `||` requires both operands to be `Bool` and returns `Bool`.

### 3.2 Precedence

Operator precedence around the new operators is:

```text
??  >  ||  >  &&  >  == != < > <= >=
```

This means:

- `false || 1 == 1` parses as `false || (1 == 1)`
- `1 == 1 && 2 == 2` parses as `(1 == 1) && (2 == 2)`

Both operators are left-associative.

### 3.3 Evaluation

In v2.1.0, `&&` and `||` are evaluated eagerly in the VM.
Short-circuit evaluation is not part of this release.

### 3.4 Errors

| Code | Description |
|---|---|
| `E070` | non-`Bool` operand used with `&&` |
| `E071` | non-`Bool` operand used with `||` |

## 4. `fav new`

CLI syntax:

```sh
fav new <name> [--template <script|pipeline|lib>]
```

Default template:

- `script`

Behavior:

- Fails if the destination path already exists.
- Fails if `--template` is not one of `script`, `pipeline`, or `lib`.
- Prints next-step guidance after creation.

### 4.1 `script` template

```text
<name>/
  fav.toml
  src/
    main.fav
```

`src/main.fav`:

```favnir
public fn main() -> Unit !Io {
    IO.println(greet("world"))
}

fn greet(name: String) -> String {
    $"Hello {name}!"
}
```

### 4.2 `pipeline` template

```text
<name>/
  fav.toml
  rune.toml
  src/
    main.fav
    pipeline.fav
    stages/
      parse.fav
      validate.fav
      save.fav
```

`src/main.fav` prints `"pipeline: ok"`.

`src/pipeline.fav` defines:

```favnir
public seq MainPipeline =
    ParseStage
    |> ValidateStage
    |> SaveStage
```

### 4.3 `lib` template

```text
<name>/
  fav.toml
  rune.toml
  src/
    lib.fav
    lib.test.fav
```

`src/lib.fav` exports `hello() -> String`.

## 5. Welcome Screen

When `fav` is run with no arguments:

- the CLI attempts to render `versions/favnir.png`
- color output is skipped if `NO_COLOR` is set
- the help text is then printed

`fav --help`, `fav -h`, and `fav help` print help text directly.

## 6. Compatibility

Favnir v2.1.0 is an additive release over v2.0.0.

- Existing v2.0.0 source remains valid.
- Removed v1.x keywords remain invalid.
- New error codes `E070` and `E071` are additive.

## 7. Validation Notes

Implementation status for this spec:

- `cargo build` passes
- `cargo test` passes
