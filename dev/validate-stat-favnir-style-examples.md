# Validate + Stat Favnir-Style Examples

## Goal

This note rewrites the `validate` + `stat` story in a more Favnir-like style.

The focus is not just API listing.
The focus is how the code should feel when written in real Favnir source.

The target style is:

- expression-oriented
- `bind`-first
- `trf` and `flw` friendly
- readable in notebooks and tests
- minimal temporary ceremony

## Imports

Recommended naming:

```fav
use stat as Stat
use validate.field as Field
use validate.db as DbValidate
use validate.flow as Flow
```

This is short, readable, and makes intent obvious.

## 1. Lightweight Value Validation

This is the most local form.
It should feel like building a reusable rule pipeline.

```fav
bind EmailRule <-
    Field.required()
    |> Field.email()
    |> Field.max_len(255)

bind AgeRule <-
    Field.range(0, 120)
```

This is simple and compositional.
It also matches Favnir's `bind` + pipeline style naturally.

## 2. Flow Validator Definition

A richer validator should still read like a pipeline asset.

```fav
type Signup = {
    email: String
    password: String
    confirm: String
}

bind SignupValidator <-
    Flow.validator<Signup>()
    |> Flow.field("email", _.email, Field.required() |> Field.email())
    |> Flow.field("password", _.password, Field.min_len(8))
    |> Flow.cross(
        "password_mismatch",
        |x| x.password == x.confirm,
        "password and confirm must match"
    )
```

This should be considered the canonical style for rich validation.

It is:

- typed
- compositional
- local
- easy to explain in a graph later

## 3. Validation as a `trf`

Validation should fit naturally into a normal flow pipeline.

```fav
trf ValidateSignup: Signup -> Signup! = |form| {
    Flow.validate(form, SignupValidator)
}
```

This should be the default production-facing pattern.

A validator becomes a reusable `trf`, not a special framework object.

## 4. Type-Driven Data Generation

Synthetic generation should feel just as direct.

```fav
type UserInput = {
    name: String
    email: String
    age: Int
}

bind users <- Stat.list<UserInput>(100, seed: 42)
```

This should be one of the flagship examples for `stat`.

It says:

- generate data directly from the type
- keep it deterministic with `seed`
- make testing and notebook work easy

## 5. Simple Validation + Synthetic Input

This is the first end-to-end pattern.

```fav
type UserInput = {
    name: String
    email: String
    age: Int
}

bind UserValidator <-
    Flow.validator<UserInput>()
    |> Flow.field("name", _.name, Field.required() |> Field.min_len(1))
    |> Flow.field("email", _.email, Field.required() |> Field.email())
    |> Flow.field("age", _.age, Field.range(0, 120))

bind users <- Stat.list<UserInput>(100, seed: 42)

bind checked <-
    users
    |> List.map(|u| Flow.validate_all(u, UserValidator))
```

This should be a canonical notebook example.

## 6. Testing Style

A Favnir test should read like a normal pipeline, not a special testing DSL.

```fav
test "generated users satisfy validation" {
    bind users <- Stat.list<UserInput>(100, seed: 42)
    bind result <- Flow.validate_all(users[0], UserValidator)

    match result {
        ok(_) => assert(true)
        err(errors) => fail(Debug.show(errors))
    }
}
```

This is important.
The story is not:

- validation framework + separate fixture tool

The story is:

- typed generation + validation + normal Favnir test

## 7. DB-Oriented Row Validation

Storage-facing validation should still look like a normal asset definition.

```fav
type UserRow = {
    name: String
    email: String
    age: Int
}

bind UserRowValidator <-
    DbValidate.record([
        DbValidate.field("name", Field.required() |> Field.min_len(1)),
        DbValidate.field("email", Field.required() |> Field.email() |> Field.max_len(255)),
        DbValidate.field("age", Field.range(0, 120))
    ])
```

This should feel lighter than `Flow`, but still pipeline-friendly.

## 8. DB Validation as a `trf`

```fav
trf ValidateRows: List<UserRow> -> List<UserRow]! = |rows| {
    rows
    |> List.map(|row| DbValidate.validate_row(row, UserRowValidator))
    |> Result.collect()
}
```

This is a strong pattern because it keeps storage-facing checks in the same shape as the rest of the language.

## 9. Full Import Flow

This is one of the most important examples for Favnir.

```fav
type UserRow = {
    name: String
    email: String
    age: Int
}

bind UserRowValidator <-
    DbValidate.record([
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
```

This is a key identity example for Favnir.

It shows:

- type-driven shape
- validation before persistence
- explicit pipeline stages
- no framework ceremony

## 10. Flow Validation for Forms or Domain Inputs

The richer `Flow` layer should feel equally natural.

```fav
trf ParseSignup: Json -> Signup! = |json| {
    DecodeSignup(json)
}

trf ValidateSignup: Signup -> Signup! = |form| {
    Flow.validate(form, SignupValidator)
}

trf SaveSignup: Signup -> UserId !Db = |form| {
    Db.execute("insert ...")
}

flw RegisterUser =
    ParseSignup
    |> ValidateSignup
    |> SaveSignup
```

This is where Favnir should differ from framework-heavy languages.
Validation is just another well-typed stage in the flow.

## 11. Sampling Real Data Before Validation

`stat` should not feel synthetic-only.
It should also help with real datasets.

```fav
bind rows <- Csv.parse_with_header(text)

bind sample <-
    rows
    |> Stat.sample_rows(100, seed: 42)

bind checked <-
    sample
    |> List.map(|row| DbValidate.validate_row(row, UserRowValidator))
```

This is a strong notebook and QA pattern.

## 12. Simulation-Friendly Pattern

A later, more advanced example could look like this.

```fav
bind rows <- Stat.simulate<UserRow>(count: 10000, model: UserRowScenario, seed: 42)
bind result <- ValidateRows(rows)
```

This should feel like an extension of the same design, not a separate subsystem.

## 13. Style Guidance

The recommended Favnir style is:

- define field rules with `bind`
- build validators with pipelines
- wrap validators in `trf` for production use
- use `validate_all` in tests and notebooks
- use `Stat.*` to generate or sample data locally

In other words:

- rules are values
- validators are values
- validation stages are `trf`
- synthetic and sampled input are just data sources

## 14. Summary

The Favnir-like way to combine `validate` and `stat` is:

- `Field` rules are small pipelineable values
- `DbValidate` and `Flow` assemble them into reusable validators
- `Stat` provides deterministic synthetic or sampled input
- validation is wrapped into ordinary `trf`
- complete data workflows are expressed as normal `flw`

This gives the feature set a natural place in the language without adding framework-style ceremony.
