# Favnir v2.2.0 Language Specification

Date: 2026-05-13

This document records the additive surface finalized in Favnir v2.2.0.

## 1. Summary

Favnir v2.2.0 formalizes:

- `|> match { ... }` pipeline matching
- pattern guards with `where`
- built-in variant-pattern normalization for `Ok`, `Err`, `Some`, and `None`

These features build on syntax already present in Favnir and complete the
runtime-facing validation and documentation for them.

## 2. Pipe Match

Favnir allows `match` in pipeline position:

```favnir
bind result <- Result.ok(5)
result |> match {
    Ok(v)  => v
    Err(_) => 0
}
```

This is parsed and compiled as an ordinary `match` expression whose scrutinee
is the pipeline input value.

Valid examples:

- `Result.ok(5) |> match { Ok(v) => v Err(_) => 0 }`
- `Result.err("oops") |> match { Ok(v) => v Err(_) => -1 }`
- `Option.some(42) |> match { Some(v) => v None => 0 }`
- `Option.none() |> match { Some(v) => v None => -1 }`
- `double(7) |> match { Ok(v) => v Err(_) => 0 }`

## 3. Pattern Guards

Favnir match arms may include an optional `where` guard:

```favnir
match value {
    n where n > 10 => "big"
    _              => "small"
}
```

Evaluation rules:

- the arm pattern is matched first
- the `where` expression is evaluated only if the pattern matched
- if the guard is `false`, evaluation continues to the next arm

Pattern guards work with:

- bind patterns
- record patterns
- variant patterns
- compound boolean conditions such as `&&` and `||`

Example:

```favnir
type User = { name: String age: Int }

match user {
    { age } where age >= 18 => "adult"
    _                       => "minor"
}
```

## 4. Built-in Variant Pattern Normalization

At runtime, built-in `Result` and `Option` values use lowercase tags:

| Value constructor | Runtime tag |
|---|---|
| `Result.ok(x)` | `"ok"` |
| `Result.err(x)` | `"err"` |
| `Option.some(x)` | `"some"` |
| `Option.none()` | `"none"` |

In source patterns, Favnir normalizes canonical built-in names during IR
compilation:

| Source pattern | Normalized IR tag |
|---|---|
| `Ok(...)` | `"ok"` |
| `Err(...)` | `"err"` |
| `Some(...)` | `"some"` |
| `None` | `"none"` |

This lets the following source work correctly:

```favnir
match result {
    Ok(v)  => v
    Err(_) => 0
}
```

User-defined ADT constructors are not rewritten.
For example, `Circle(r)` remains `Circle`.

## 5. Errors

Favnir v2.2.0 adds no new error codes.

Relevant existing error:

| Code | Description |
|---|---|
| `E027` | match guard expression must be `Bool` |

Example:

```favnir
fn f(x: Int) -> Int {
    match x {
        n where n + 1 => n
        _ => 0
    }
}
```

This is rejected with `E027`.

## 6. Compatibility

Favnir v2.2.0 is additive over v2.1.0.

- Existing v2.1.0 code remains valid.
- `pipe match` remains a desugaring to ordinary `match`.
- Built-in variant normalization affects only `Ok`, `Err`, `Some`, and `None`.
- User ADT constructor matching behavior is unchanged.

## 7. Validation Notes

Implementation status for this spec:

- `cargo build` passes
- `cargo test` passes
