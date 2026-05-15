# Favnir v3.0.0 Language Specification

## Overview

Favnir v3.0.0 is a pipeline-first, purely-functional DSL targeting data engineers.
It features a strong type system, algebraic effects, and a self-hosting compiler milestone.

---

## 1. Lexical Structure

### 1.1 Comments
```
// single-line comment
```
Block comments are not supported.

### 1.2 Keywords

| Group   | Keywords |
|---------|---------|
| Control | `if` `else` `match` |
| Binding | `bind` `fn` `public` |
| Types   | `type` `interface` `impl` |
| Pipeline| `stage` `seq` |
| Iteration | `for` `in` `collect` `yield` |
| Async   | `async` `effect` |
| Literals | `true` `false` |
| Misc    | `abstract` `import` `as` `with` `test` `bench` |

> **Note**: `trf` and `flw` are reserved as deprecated aliases (E0901/E0902).
> Use `stage` and `seq` exclusively. Run `fav migrate` to upgrade legacy code.

### 1.3 Operators

| Operator | Meaning |
|----------|---------|
| `->` | Function arrow / return type |
| `<-` | Bind arrow |
| `\|>` | Pipe (stage application) |
| `=>` | Match arm |
| `==` `!=` | Equality |
| `<=` `>=` `<` `>` | Comparison |
| `&&` `\|\|` | Boolean logic |
| `??` | Null-coalesce (Option unwrap with default) |
| `::` | Namespace access |
| `\|` | Union / pipe literal |

---

## 2. Types

### 2.1 Primitive Types

| Type   | Example |
|--------|---------|
| `Int`  | `42`, `-7` |
| `Float`| `3.14`, `-0.5` |
| `Bool` | `true`, `false` |
| `String`| `"hello"`, `$"Hi {name}!"` |
| `Unit` | `()` |

### 2.2 Composite Types

```favnir
// Record type
type Point = {
    x: Int
    y: Int
}

// Variant (enum) type
type Shape =
    | Circle(Float)
    | Rect(Float, Float)
    | Empty

// Generic type alias
type Pair<A, B> = { first: A  second: B }
```

### 2.3 Generic / Parameterized Types

| Type | Description |
|------|-------------|
| `List<T>` | Immutable linked list |
| `Option<T>` | `Some(value)` or `None` |
| `Result<T, E>` | `Ok(value)` or `Err(error)` |
| `Task<T>` | Async computation |
| `Stream<T>` | Lazy infinite or finite sequence |

### 2.4 Effect Signatures

Effects are declared with `!` after the return type:

```favnir
fn read_file(path: String) -> String !Io
fn fetch(url: String) -> String !Network
fn query(sql: String) -> List<Row> !Db
```

---

## 3. Expressions

### 3.1 Bind

```favnir
bind name <- expr
```

Binds the result of `expr` to `name` in the lexical scope of the following expression.

### 3.2 If-Else

```favnir
if condition { then_expr } else { else_expr }
```

Both branches must have the same type. `else if` is not a keyword — use nested if-else.

### 3.3 Match

```favnir
match value {
    Pattern1 => expr1
    Pattern2 => expr2
    _        => default_expr
}
```

Patterns: variant constructors, record destructuring `{ field }`, literal values, wildcard `_`.

### 3.4 Collect / Yield

```favnir
collect {
    yield item1;
    yield item2;
    for x in list { yield x; }
}
```

Produces a `List<T>`.

### 3.5 For-In

```favnir
for x in list {
    body_expr;
}
```

Evaluates to `Unit`. Desugars to `List.fold`. Use `collect { for x in list { yield f(x); } }` to transform.

### 3.6 Null-Coalesce

```favnir
opt_value ?? default_value
```

Equivalent to `Option.unwrap_or(opt_value, default_value)`.

### 3.7 String Interpolation

```favnir
$"Hello, {name}! You are {age} years old."
```

All interpolated expressions must be `String`; use `Debug.show` or `String.from_int` to convert.

### 3.8 Pipe

```favnir
value |> StageName
```

Applies a `stage` to a value. Can be chained: `x |> A |> B |> C`.

---

## 4. Declarations

### 4.1 Functions

```favnir
fn add(x: Int, y: Int) -> Int {
    x + y
}

public fn main() -> Unit !Io {
    IO.println("hello")
}

public async fn fetch_data() -> List<Row> !Network {
    // ...
}
```

### 4.2 Stage (formerly `trf`)

```favnir
stage Double: Int -> Int = |x| { x * 2 }

abstract stage Parse: String -> Int
```

### 4.3 Seq (formerly `flw`)

```favnir
seq Pipeline = Stage1 |> Stage2 |> Stage3
```

### 4.4 Type Definitions

```favnir
public type User = {
    id:   Int
    name: String
    age:  Int
}

type Status = | Active | Inactive | Pending
```

### 4.5 Interface

```favnir
interface Show {
    fn show(self: Self) -> String
}

impl Show for User {
    show = |u| $"User(${u.id}, ${u.name})"
}
```

### 4.6 Test / Bench

```favnir
test "description" {
    assert_eq(1 + 1, 2)
}

bench "description" {
    1 + 1
}
```

---

## 5. Standard Library

### 5.1 IO

| Function | Signature |
|----------|-----------|
| `IO.println` | `String -> Unit !Io` |
| `IO.print` | `String -> Unit !Io` |
| `IO.read_line` | `() -> String !Io` |

### 5.2 List

| Function | Signature |
|----------|-----------|
| `List.map` | `(List<A>, A -> B) -> List<B>` |
| `List.filter` | `(List<A>, A -> Bool) -> List<A>` |
| `List.fold` | `(List<A>, B, (B, A) -> B) -> B` |
| `List.first` | `List<A> -> Option<A>` |
| `List.last` | `List<A> -> Option<A>` |
| `List.length` | `List<A> -> Int` |
| `List.concat` | `(List<A>, List<A>) -> List<A>` |
| `List.range` | `(Int, Int) -> List<Int>` |
| `List.take` / `List.drop` | `(List<A>, Int) -> List<A>` |
| `List.zip` | `(List<A>, List<B>) -> List<(A,B)>` |
| `List.join` | `(List<String>, String) -> String` |
| `List.sort` | `(List<A>, (A, A) -> Int) -> List<A>` |
| `List.find` / `List.any` / `List.all` | predicate operations |

### 5.3 Option

| Function | Signature |
|----------|-----------|
| `Option.unwrap_or` | `(Option<A>, A) -> A` |
| `Option.map` | `(Option<A>, A -> B) -> Option<B>` |
| `Option.is_some` / `Option.is_none` | `Option<A> -> Bool` |

### 5.4 String

| Function | Signature |
|----------|-----------|
| `String.length` | `String -> Int` |
| `String.concat` | `(String, String) -> String` |
| `String.slice` | `(String, Int, Int) -> String` |
| `String.char_at` | `(String, Int) -> Option<String>` |
| `String.contains` | `(String, String) -> Bool` |
| `String.split` | `(String, String) -> List<String>` |
| `String.trim` | `String -> String` |
| `String.starts_with` / `String.ends_with` | predicates |

### 5.5 Stream

| Function | Signature |
|----------|-----------|
| `Stream.of` | `List<A> -> Stream<A>` |
| `Stream.from` | `List<A> -> Stream<A>` |
| `Stream.gen` | `(A, A -> A) -> Stream<A>` (infinite) |
| `Stream.map` | `(Stream<A>, A -> B) -> Stream<B>` |
| `Stream.filter` | `(Stream<A>, A -> Bool) -> Stream<A>` |
| `Stream.take` | `(Stream<A>, Int) -> Stream<A>` |
| `Stream.to_list` | `Stream<A> -> List<A>` |

---

## 6. Error Codes

See `fav explain-error --list` for a complete listing.
See `fav explain-error <code>` for details on a specific code.

| Range | Category |
|-------|----------|
| E01xx | Syntax / structure |
| E02xx | Type errors |
| E03xx | Effect errors |
| E05xx | Module / import errors |
| E06xx | Semantic / iteration errors |
| E07xx | Match errors |
| E09xx | Deprecated keyword errors |

---

## 7. Compilation Pipeline

See `fav explain compiler` for an interactive summary.

| Step | Module | Description |
|------|--------|-------------|
| 1. Lex | `frontend/lexer.rs` | Source → token stream |
| 2. Parse | `frontend/parser.rs` | Tokens → typed AST |
| 3. Check | `middle/checker.rs` | Type and effect checking |
| 4. Compile | `middle/compiler.rs` | AST → IR (desugaring) |
| 5. Codegen | `backend/codegen.rs` or `wasm_codegen.rs` | IR → .fvc or .wasm |

---

## 8. Self-Hosting Milestone (v3.0.0)

`selfhost/lexer/lexer.fav` — Complete Favnir lexer written in Favnir.
`selfhost/lexer/lexer.test.fav` — 71 tests; all passing.

`selfhost/parser/ast.fav` — AST type definitions.
`selfhost/parser/parser.fav` — Recursive-descent expression parser (arithmetic).
`selfhost/parser/parser.test.fav` — 18 tests; all passing.
`selfhost/parser/main.fav` — Combined lex+parse demo.

Design constraints:
- No recursive types: AST encoded as S-expression strings.
- Nesting depth ≤ 23 per function (Rust checker stack limit on Windows).
- Recursion requires 64 MB thread stack (`main.rs` spawns with `stack_size(64MB)`).

---

## 9. Toolchain Commands

| Command | Description |
|---------|-------------|
| `fav run [file]` | Run a Favnir program |
| `fav check [file]` | Type-check only |
| `fav build [-o out] [--target fvc\|wasm] [file]` | Build artifact |
| `fav exec <artifact>` | Run compiled .fvc |
| `fav test [file]` | Run test blocks |
| `fav bench [file]` | Run bench blocks |
| `fav explain [file]` | Show API signatures |
| `fav explain compiler` | Show compilation pipeline |
| `fav explain diff <from> <to>` | Diff explain metadata |
| `fav explain-error <code>` | Show error details |
| `fav explain-error --list` | List all error codes |
| `fav migrate [file]` | Upgrade v1.x → v2.0+ |
| `fav fmt [file]` | Format source |
| `fav lint [file]` | Lint checks |
| `fav graph [file]` | Show stage/fn dependency graph |
| `fav bundle [file]` | Build trimmed artifact + manifest |
| `fav watch [file]` | Watch-mode re-run |
| `fav new <name>` | Create new project |
| `fav install` | Install dependencies |
| `fav lsp` | Start LSP server |
