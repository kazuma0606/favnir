# Validate Rune Architecture

## Positioning

Favnir should treat validation as an official rune family rather than a single built-in language feature.

The recommended structure is:

- `validate`
- `validate.field`
- `validate.db`
- `validate.flow`

This keeps lightweight field checks and full pipeline validation separate while preserving a shared error model.

## Roles

### `validate`

The root rune contains only shared concepts.

Suggested contents:

- `ValidationError`
- `ValidationResult<T>`
- shared path/code/message helpers

Example:

```fav
type ValidationError = {
    path: String
    code: String
    message: String
}
```

This layer should stay small and stable.

### `validate.field`

This rune handles lightweight value- and field-level validation.

Typical responsibilities:

- `required`
- `min_len`
- `max_len`
- `range`
- `email`
- `one_of`
- `custom`

This layer is reusable from both database-oriented and flow-oriented validation.

Example direction:

```fav
use validate.field as Field

bind email_rule <-
    Field.required()
    |> Field.email()
    |> Field.max_len(255)
```

`validate.field` is the lowest reusable validation layer.

### `validate.db`

This rune is for database-column and row-oriented validation.

It should not try to replace the database constraint system itself.
Instead, it should validate records before insert/update/import.

Typical responsibilities:

- nullable / required checks
- string length checks for columns
- numeric range / precision / scale checks
- row-oriented validator assembly
- CSV/import preflight checks

Example direction:

```fav
use validate.field as Field
use validate.db as DbValidate

bind UserRowValidator <- DbValidate.record([
    DbValidate.field("email", Field.required() |> Field.email()),
    DbValidate.field("age", Field.range(0, 120))
])
```

This rune is intended for ingestion boundaries and storage-facing validation.

### `validate.flow`

This rune is the richer pipeline/form/domain validation layer.

Typical responsibilities:

- field validation using extractors
- cross-field validation
- conditional validation
- nested validation
- each-item validation
- fail-fast validation
- collect-all validation

Example direction:

```fav
use validate.field as Field
use validate.flow as Flow

bind SignupValidator <- Flow.validator<Signup>()
    |> Flow.field("email", _.email, Field.required() |> Field.email())
    |> Flow.field("password", _.password, Field.min_len(8))
    |> Flow.cross(
        "password_mismatch",
        |x| x.password == x.confirm,
        "password and confirm must match"
    )
```

Execution style:

```fav
Flow.validate(form, SignupValidator)
Flow.validate_all(form, SignupValidator)
```

This is the layer that corresponds most closely to Forge's richer `Validator::new(...).validate_all(...)` model.

## Dependency Direction

The dependency structure should stay simple.

- `validate`
  - base types only
- `validate.field`
  - depends on `validate`
- `validate.db`
  - depends on `validate`
  - depends on `validate.field`
- `validate.flow`
  - depends on `validate`
  - depends on `validate.field`

This keeps `field` reusable and prevents `db` and `flow` from leaking concerns into each other.

## API Shape

A good first-pass shape is:

### Base

```fav
type ValidationError = {
    path: String
    code: String
    message: String
}
```

### Flow entrypoints

```fav
Flow.validate<T>(value: T, validator: Validator<T>) -> T!
Flow.validate_all<T>(value: T, validator: Validator<T>) -> Result<T, List<ValidationError>>
```

### Field examples

```fav
Field.required() -> Rule<String?>
Field.min_len(n: Int) -> Rule<String>
Field.max_len(n: Int) -> Rule<String>
Field.range(min: Int, max: Int) -> Rule<Int>
Field.email() -> Rule<String>
```

### Flow examples

```fav
Flow.field<T, F>(name: String, get: T -> F, rule: Rule<F>) -> Validator<T>
Flow.cross<T>(code: String, check: T -> Bool, message: String) -> Validator<T>
Flow.nested<T, F>(name: String, get: T -> F, validator: Validator<F>) -> Validator<T>
Flow.each<T, F>(name: String, get: T -> List<F>, validator: Validator<F>) -> Validator<T>
Flow.when<T>(pred: T -> Bool, inner: Validator<T>) -> Validator<T>
```

### DB examples

```fav
DbValidate.field(name: String, rule: Rule<T>) -> DbFieldRule
DbValidate.record(rules: List<DbFieldRule>) -> DbValidator
DbValidate.validate_row(row: Map<String>, validator: DbValidator) -> Result<Map<String>, List<ValidationError>>
```

Exact names can still change, but the layering should remain.

## Why This Fits Favnir

This architecture matches Favnir better than a decorator-heavy design.

Reasons:

- keeps validation compositional
- works naturally with `trf` and `flw`
- keeps fail-fast and collect-all explicit
- avoids embedding HTTP or middleware assumptions into the language core
- is suitable as a pure Favnir library family

It also creates a strong milestone:

- `validate` can be the first official Pure Favnir rune family that is not fundamentally Rust-dependent

## Recommended Implementation Order

1. `validate`
   - `ValidationError`
   - shared result helpers
2. `validate.field`
   - `required`
   - `min_len`
   - `max_len`
   - `range`
   - `email`
3. `validate.flow`
   - `field`
   - `cross`
   - `validate`
   - `validate_all`
4. `validate.db`
   - row/column wrappers on top of `field`

## Summary

The recommended direction is:

- `validate` = parent rune
- `validate.field` = lightweight reusable field/value validation
- `validate.db` = storage/import-oriented validation
- `validate.flow` = richer pipeline/form/domain validation

This gives Favnir both lightweight and full validation without overloading the language core.
