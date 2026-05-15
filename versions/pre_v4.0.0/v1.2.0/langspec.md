# Favnir v1.2.0 Langspec

This version adds two core features:

- `invariant` inside record-like `type` definitions
- `std.states`, a virtual standard module of common validated state types

## Invariant Syntax

`invariant` appears inside a record `type` body after fields.

```favnir
type Email = {
    value: String
    invariant String.contains(value, "@")
    invariant String.length(value) > 3
}
```

Each invariant is checked in field scope. Multiple invariants are combined with logical `AND`.

`invariant` is supported only for record-style types in v1.2.0.

## Invariant Type Rules

- Each invariant expression must type-check to `Bool`
- Unknown field references are ordinary type errors
- Invariants are pure checks and do not introduce new effects

### Error Codes

- `E045`: invariant expression did not type-check to `Bool`
- `E046`: invariant state constructor or typed bind failed at compile time for a literal value

## Typed Bind Sugar

Typed bind can target invariant-bearing record types:

```favnir
bind age: PosInt <- 25
```

In v1.2.0 this is invariant-aware sugar for validation against the state type. Literal inputs may fail at compile time with `E046`.

## Synthetic Constructors

For every invariant-bearing record type, Favnir synthesizes:

```favnir
TypeName.new(field1, field2, ...) -> TypeName!
```

The constructor:

- builds the record value
- checks all invariants
- returns `ok(value)` on success
- returns `err("InvariantViolation: TypeName")` on failure

Example:

```favnir
type PosInt = {
    value: Int
    invariant value > 0
}

bind good <- PosInt.new(5)
bind bad  <- PosInt.new(-1)
```

## std.states

`std.states` is a virtual standard module that exports common validated state types.

Available types in v1.2.0:

- `PosInt`
- `NonNegInt`
- `Probability`
- `PortNumber`
- `NonEmptyString`
- `Email`
- `Url`
- `Slug`

Examples:

```favnir
use std.states.Email
use std.states.PosInt

bind age <- PosInt.new(25)
bind email <- Email.new("user@example.com")
```

## explain

`fav explain` includes an `INVARIANTS` column for record types with invariant checks.

`fav explain --schema` lowers supported invariant expressions to SQL `CHECK` constraints and comments unsupported expressions.
