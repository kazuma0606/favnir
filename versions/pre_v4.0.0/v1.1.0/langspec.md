# Favnir v1.1.0 Language Spec

Date: 2026-05-06

This document fixes the surface language added in v1.1.0:

- `interface`
- `impl Interface for Type`
- `type ... with ...`
- built-in interfaces `Show`, `Eq`, `Ord`, `Gen`, `Semigroup`, `Monoid`, `Group`, `Ring`, `Field`
- deprecation warning `W010` for legacy `cap`

It is additive to the v1.0.0 language.

## 1. Interface Declarations

An interface declares method signatures over `Self`.

```fav
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {
    compare: Self -> Self -> Int
}
```

Rules:

- `interface Name { ... }` declares a new interface.
- `interface Name : Super { ... }` declares one direct super-interface.
- `Self` may appear only inside interface method signatures.
- Method bodies are not allowed in `interface`.

## 2. Interface Implementations

An implementation binds one or more interfaces to a concrete type.

```fav
impl Show for Point {
    show = |p| "Point"
}

impl Show, Eq for UserRow
```

Rules:

- `impl Interface for Type { ... }` is a manual implementation.
- `impl A, B for Type { ... }` implements multiple interfaces in one declaration.
- `impl Interface for Type` without a body requests auto-synthesis.
- Auto-synthesis succeeds only when the target type structurally supports the requested interface.

Current runtime call shape is namespace-like:

```fav
Point.show.show(p)
Int.ord.compare(a, b)
Float.field.divide(x, y)
```

The first segment is the type name. The second segment is the lowercased interface namespace. The third segment is the method name.

## 3. `with` Sugar

Record types may request auto-generated interface implementations directly on the type declaration.

```fav
type UserRow with Show, Eq = {
    name: String
    age: Int
}
```

This is equivalent in intent to:

```fav
type UserRow = {
    name: String
    age: Int
}

impl Show, Eq for UserRow
```

## 4. Built-in Interfaces

### 4.1 Show / Eq / Ord

Built-in interface definitions:

```fav
interface Show {
    show: Self -> String
}

interface Eq {
    eq: Self -> Self -> Bool
}

interface Ord : Eq {
    compare: Self -> Self -> Int
}
```

Built-in implementations:

- `Int`: `Show`, `Eq`, `Ord`
- `Float`: `Show`, `Eq`, `Ord`
- `Bool`: `Show`, `Eq`
- `String`: `Show`, `Eq`, `Ord`

### 4.2 Gen

`Gen` is the minimal generation interface used by later `Stat` integration.

```fav
interface Gen {
    gen: Int? -> Self
}
```

Built-in implementations:

- `Int`
- `Float`
- `Bool`
- `String`

Auto-synthesis of `Gen` is allowed for record types when all fields implement `Gen`.

### 4.3 Algebraic Interfaces

```fav
interface Semigroup {
    combine: Self -> Self -> Self
}

interface Monoid : Semigroup {
    empty: () -> Self
}

interface Group : Monoid {
    inverse: Self -> Self
}

interface Ring : Monoid {
    multiply: Self -> Self -> Self
}

interface Field : Ring {
    divide: Self -> Self -> Result<Self, Error>
}
```

Built-in implementations:

- `Int`: `Semigroup`, `Monoid`, `Group`, `Ring`
- `Float`: `Semigroup`, `Monoid`, `Group`, `Ring`, `Field`

## 5. Static Checking

The checker enforces:

- unknown interface in `impl` => `E041`
- method type mismatch => `E042`
- missing required super-interface impl => `E043`
- auto-synthesis impossible because a field lacks the required interface => `E044`

Examples:

```fav
impl UnknownIface for Int { ... }   -- E041
impl Show for Int { show = |x| x }  -- E042
impl Ord for UserRow { ... }        -- E043 if Eq is missing
impl Show for BadRow                -- E044 if a field cannot Show
```

Interfaces may also appear as explicit parameters:

```fav
fn sort<T>(items: List<T>, ord: Ord<T>) -> List<T> {
    items
}
```

## 6. Legacy `cap`

Legacy `cap` and old-style `impl Eq<Int> { ... }` still compile in v1.1.0 for compatibility, but they are deprecated.

Checker warning:

```text
W010: `cap` is deprecated. Use `interface` instead.
```

Behavior:

- `fav check` prints `W010`
- `fav check --no-warn` suppresses `W010`
- old `cap` examples still compile and run in v1.1.0

## 7. Examples

Reference examples shipped with v1.1.0:

- `examples/interface_basic.fav`
- `examples/interface_auto.fav`
- `examples/algebraic.fav`

## 8. Done Definition Snapshot

v1.1.0 is considered complete when all of the following hold:

- `interface Show { show: Self -> String }` parses and checks
- `impl Show for Int { ... }` parses and checks
- `impl Show, Eq for UserRow` works as auto-synthesis
- `type UserRow with Show, Eq = { ... }` works as sugar
- `Gen` and `Field` families are registered as built-in interfaces
- legacy `cap` emits `W010` but still compiles
