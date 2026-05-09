# Favnir Language Specification — v1.9.0

## Overview

v1.9.0 adds ergonomic iteration with `for`, null-safe access with `??`, keyword aliases `stage`/`seq`, and tooling improvements (coverage HTML, bench statistics).

---

## 1. `for x in list { body }` — For-In Expression

### Syntax

```
for <ident> in <expr> { <stmt>* }
```

### Semantics

Iterates over a `List<T>`, binding each element to `<ident>` and executing `<body>` for side effects. The expression evaluates to `Unit`.

Desugars to `List.fold(list, Unit, |$acc, x| { body; Unit })`.

### Type rules

- `<expr>` must be `List<T>` — error **E065** otherwise
- Body must not contain bare `yield` at the top level (use `collect` instead) — **E066**
- `for` is not allowed directly inside a `collect { }` block — error **E067**
- The iteration variable `<ident>` is scoped to the body block

### Example

```favnir
bind nums <- collect { yield 1; yield 2; yield 3; }
for n in nums {
    IO.println_int(n)
}
```

### Error Codes

| Code | Condition |
|------|-----------|
| E065 | Iterator expression is not `List<T>` |
| E066 | Body contains `yield` without enclosing `collect` |
| E067 | `for` used directly inside a `collect` block |

---

## 2. `??` — Null-Coalesce Operator

### Syntax

```
<expr> ?? <expr>
```

### Semantics

Lowest-precedence binary operator. If the left-hand side is `Option.some(v)`, evaluates to `v`. If it is `Option.none()`, evaluates to the right-hand side.

Desugars to `Option.unwrap_or(lhs, rhs)`.

### Type rules

- LHS must be `Option<T>` — error **E068** otherwise
- RHS must be compatible with `T` — error **E069** if type mismatch
- Result type is `T`

### Precedence

`??` has lower precedence than all arithmetic and comparison operators, higher than pipeline `|>`.

### Example

```favnir
bind name <- find_user(id) ?? "Unknown"
bind n    <- String.to_int(s) ?? 0
```

### Error Codes

| Code | Condition |
|------|-----------|
| E068 | LHS of `??` is not `Option<T>` |
| E069 | RHS type does not match the inner type `T` |

---

## 3. `stage` / `seq` — Keyword Aliases

`stage` is an alias for `trf` (pipeline transform stage).
`seq` is an alias for `flw` (pipeline flow/sequence).

Both keywords are fully interchangeable with their counterparts — same parsing, same semantics, same type system rules.

### Syntax

```favnir
// trf / stage
stage <name>: <InputType> -> <OutputType> [!Effects] = |params| { body }

// flw / seq
seq <name> = <stage1> [|> <stage2> ...]
```

### Example

```favnir
stage double: Int -> Int = |x| { x * 2 }
seq pipeline = double

public fn main() -> Int {
    5 |> pipeline   // 10
}
```

### Notes

- No deprecation warnings for `trf`/`flw` in v1.9.0 — both forms coexist
- `stage`/`seq` are reserved keywords as of v1.9.0
- Migration to `stage`/`seq` will be automated via `fav migrate` in a future release

---

## 4. Coverage HTML Output (tooling)

`fav test --coverage-report <dir>` now generates:

- `<dir>/coverage.txt` — plain-text line-level coverage (existing)
- `<dir>/index.html` — HTML summary with per-file coverage percentages
- `<dir>/<filename>.html` — per-file source-annotated HTML with covered/uncovered line highlighting

Color coding:
- Green background: covered line
- Red background: uncovered line
- Gray: non-trackable line (comments, blank)

---

## 5. Bench Statistics (tooling)

`fav bench` now reports extended statistics per benchmark:

| Metric | Description |
|--------|-------------|
| min    | Fastest iteration (ns) |
| max    | Slowest iteration (ns) |
| mean   | Arithmetic mean (ns) |
| p50    | Median (50th percentile) |
| stddev | Standard deviation |

Flags:
- `--compact` — single-line output per benchmark
- `--json` — JSON output for CI integration

---

## 6. Error Code Summary (v1.9.0 additions)

| Code | Phase | Description |
|------|-------|-------------|
| E065 | Check | `for` iterator is not `List<T>` |
| E066 | Check | `yield` in `for` body without `collect` |
| E067 | Check | `for` directly inside `collect` block |
| E068 | Check | LHS of `??` is not `Option<T>` |
| E069 | Check | RHS of `??` type mismatch with `Option` inner type |

---

## Compatibility

- All v1.8.0 programs are valid v1.9.0 programs
- `stage`/`seq` are new reserved keywords — existing identifiers named `stage` or `seq` will require renaming
- No changes to `.fvc` artifact format or opcode set
