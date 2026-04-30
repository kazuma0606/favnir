# Validate + Stat Integration API

## Goal

This note defines a concrete integration shape between:

- `validate`
- `validate.field`
- `validate.db`
- `validate.flow`
- `stat`

The goal is to make it easy to:

- build pipelines before real data exists
- generate reproducible synthetic input
- validate rows, forms, and pipeline boundaries
- use the same patterns in tests and Veltra notebooks

## Design Principle

`stat` should generate data.
`validate` should prove contracts over that data.

The combination should feel natural in `trf`, `flw`, and `test`.

## Shared Assumptions

### Validation output

```fav
type ValidationError = {
    path: String
    code: String
    message: String
}
```

### Flow validation entrypoints

```fav
Flow.validate<T>(value: T, validator: Validator<T>) -> T!
Flow.validate_all<T>(value: T, validator: Validator<T>) -> Result<T, List<ValidationError>>
```

### Structured generation

```fav
Stat.one<T>(seed: Int?) -> T
Stat.list<T>(count: Int, seed: Int?) -> List<T>
Stat.rows<T>(count: Int, seed: Int?) -> List<T>
```

## 1. Field Validation + Type-Driven Data

This is the simplest and most important combination.

```fav
use stat as Stat
use validate.field as Field
use validate.flow as Flow

type UserInput = {
    name: String
    email: String
    age: Int
}

bind UserValidator <- Flow.validator<UserInput>()
    |> Flow.field("name", _.name, Field.required() |> Field.min_len(1))
    |> Flow.field("email", _.email, Field.required() |> Field.email())
    |> Flow.field("age", _.age, Field.range(0, 120))
```

Synthetic data:

```fav
bind users <- Stat.list<UserInput>(100, seed: 42)
```

Validation:

```fav
bind checked <- users |> List.map(|u| Flow.validate_all(u, UserValidator))
```

This is the baseline integration pattern.

## 2. Fail-Fast Pipeline Validation

For real pipeline use, a `trf` should usually validate once and then continue.

```fav
trf ValidateUserInput: UserInput -> UserInput! = |input| {
    Flow.validate(input, UserValidator)
}

flw ImportUsers =
    ParseUsers
    |> ValidateUserInput
    |> SaveUsers
```

This is the simplest production-facing pattern.

## 3. Collect-All Validation for Development and QA

During development, notebook work, or diagnostics, collect-all is often better.

```fav
bind result <- Flow.validate_all(input, UserValidator)

match result {
    ok(value) => value
    err(errors) => Debug.show(errors)
}
```

This should be the preferred notebook-facing mode.

- `validate` for runtime/business flow
- `validate_all` for diagnostics and UX

## 4. DB-Oriented Validation

Database validation should wrap `Field` rules rather than inventing a totally separate system.

```fav
use validate.field as Field
use validate.db as DbValidate

type UserRow = {
    name: String
    email: String
    age: Int
}

bind UserRowValidator <- DbValidate.record([
    DbValidate.field("name", Field.required() |> Field.min_len(1)),
    DbValidate.field("email", Field.required() |> Field.email() |> Field.max_len(255)),
    DbValidate.field("age", Field.range(0, 120))
])
```

Synthetic rows:

```fav
bind rows <- Stat.rows<UserRow>(1000, seed: 7)
```

Validation:

```fav
bind checked <- rows |> List.map(|row| DbValidate.validate_row(row, UserRowValidator))
```

This is useful for:

- import pipelines
- CSV staging
- pre-insert verification
- connector-free testing

## 5. End-to-End Import Example

This is the strongest combined story.

```fav
use stat as Stat
use validate.field as Field
use validate.db as DbValidate

type UserRow = {
    name: String
    email: String
    age: Int
}

bind UserRowValidator <- DbValidate.record([
    DbValidate.field("name", Field.required() |> Field.min_len(1)),
    DbValidate.field("email", Field.required() |> Field.email()),
    DbValidate.field("age", Field.range(0, 120))
])

trf ParseRows: List<UserRow> -> List<UserRow> = |rows| {
    rows
}

trf ValidateRows: List<UserRow> -> List<UserRow]! = |rows| {
    rows
    |> List.map(|row| DbValidate.validate_row(row, UserRowValidator))
    |> Result.collect()
}

trf SaveRows: List<UserRow> -> Int !Db = |rows| {
    Db.execute("insert ...")
}

flw ImportUsers =
    ParseRows
    |> ValidateRows
    |> SaveRows

test "import pipeline works with synthetic rows" {
    bind rows <- Stat.list<UserRow>(100, seed: 99)
    bind result <- ValidateRows(rows)
    match result {
        ok(valid_rows) => assert(List.size(valid_rows) == 100)
        err(errors) => fail(Debug.show(errors))
    }
}
```

This is a strong Favnir/Veltra story because it supports:

- no real source data required
- deterministic testing
- validation before persistence
- direct pipeline integration

## 6. Cross-Field Validation + Synthetic Data

This is where `validate.flow` becomes meaningfully different from `validate.db`.

```fav
use stat as Stat
use validate.field as Field
use validate.flow as Flow

type Signup = {
    email: String
    password: String
    confirm: String
}

bind SignupValidator <- Flow.validator<Signup>()
    |> Flow.field("email", _.email, Field.required() |> Field.email())
    |> Flow.field("password", _.password, Field.min_len(8))
    |> Flow.cross(
        "password_mismatch",
        |x| x.password == x.confirm,
        "password and confirm must match"
    )
```

Synthetic forms:

```fav
bind forms <- Stat.list<Signup>(50, seed: 7)
```

Validation:

```fav
bind checked <- forms |> List.map(|f| Flow.validate_all(f, SignupValidator))
```

This is the domain/form layer, not the DB layer.

## 7. Sampling Existing Data Before Validation

`stat` is not only synthetic generation.
It should also help when real data exists but only subsets are needed.

```fav
bind rows <- Csv.parse_with_header(text)
bind sample <- Stat.sample_rows(rows, 100, seed: 42)
bind checked <- sample |> List.map(|row| DbValidate.validate_row(row, UserRowValidator))
```

This is especially useful in:

- notebook exploration
- large CSV preview
- debugging production datasets
- repeatable QA subsets

## 8. Simulation-Driven Validation

Later, `stat` should support model-driven generation.

```fav
bind rows <- Stat.simulate<UserRow>(
    count: 10000,
    model: UserRowScenario,
    seed: 42
)

bind result <- rows |> ValidateRows
```

This is useful for:

- rare event testing
- skewed distributions
- stress testing validation logic
- realistic synthetic loads

Monte Carlo belongs here as a simulation mode, not as a replacement for basic sampling.

## 9. Recommended Naming Conventions

For clarity in source code:

```fav
use stat as Stat
use validate.field as Field
use validate.db as DbValidate
use validate.flow as Flow
```

This gives a clear separation of roles:

- `Stat` = generate / sample
- `Field` = local value rules
- `DbValidate` = storage-facing row checks
- `Flow` = rich pipeline/domain validation

## 10. Recommended Minimal Public API

### `stat`

```fav
Stat.one<T>(seed: Int?) -> T
Stat.list<T>(count: Int, seed: Int?) -> List<T>
Stat.rows<T>(count: Int, seed: Int?) -> List<T>
Stat.sample_rows<T>(rows: List<T>, n: Int, seed: Int?) -> List<T>
Stat.normal(mean: Float, stddev: Float, seed: Int?) -> Float
Stat.uniform(min: Float, max: Float, seed: Int?) -> Float
```

### `validate.field`

```fav
Field.required() -> Rule<String?>
Field.min_len(n: Int) -> Rule<String>
Field.max_len(n: Int) -> Rule<String>
Field.range(min: Int, max: Int) -> Rule<Int>
Field.email() -> Rule<String>
```

### `validate.db`

```fav
DbValidate.field(name: String, rule: Rule<T>) -> DbFieldRule
DbValidate.record(rules: List<DbFieldRule>) -> DbValidator
DbValidate.validate_row<T>(row: T, validator: DbValidator) -> Result<T, List<ValidationError>>
```

### `validate.flow`

```fav
Flow.validator<T>() -> Validator<T>
Flow.field<T, F>(name: String, get: T -> F, rule: Rule<F>) -> Validator<T>
Flow.cross<T>(code: String, check: T -> Bool, message: String) -> Validator<T>
Flow.validate<T>(value: T, validator: Validator<T>) -> T!
Flow.validate_all<T>(value: T, validator: Validator<T>) -> Result<T, List<ValidationError>>
```

## 11. Product Value

This integration is not just a library convenience.
It is part of Favnir's product identity.

It enables:

- pipeline-first development
- typed synthetic data generation
- deterministic test inputs
- connector-free prototyping
- notebook-first exploration in Veltra

This is a strong differentiator compared with languages where validation and sample generation are scattered across unrelated frameworks.

## 12. Summary

Recommended direction:

- `stat` generates and samples data
- `validate.field` defines local rules
- `validate.db` checks storage-facing shapes
- `validate.flow` checks rich domain/pipeline contracts
- all four should compose naturally through `trf`, `flw`, `match`, and `test`

Together, they create a compelling Pure Favnir story for testing, ETL, and notebook-driven data workflows.
