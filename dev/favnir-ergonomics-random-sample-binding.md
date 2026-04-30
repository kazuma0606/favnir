# Favnir Developer Ergonomics Notes

## Scope

This note groups three related ergonomics topics:

- local binding inside `match`
- local binding inside blocks and conditional expressions
- synthetic data generation via `random` / `sample` runes

These are different features, but they all improve the same thing:

- writing pipelines before production data exists
- keeping intermediate variables local and readable
- making tests and notebook experimentation lightweight

## 1. Local Binding in `match`

Favnir should strongly support pattern-local binding inside `match`.

This is already aligned with the language direction.

Example:

```fav
match session {
    Guest => "guest"
    Authenticated { user } => user.email
}
```

This is the cleanest form of local binding:

- bindings are created by pattern matching
- scope stays local to the arm
- no variables leak outside the arm

This should be treated as core, not sugar.

### Recommendation

Keep and strengthen this model:

- `match` arms introduce bindings through patterns
- those bindings exist only inside the selected arm
- pattern binding should work for:
  - record patterns
  - variant patterns
  - wildcard patterns
  - nested patterns later

## 2. Local Binding Inside Blocks

Favnir should allow `bind` inside expression blocks and keep those bindings local to the block.

Example:

```fav
bind result <- {
    bind rows <- Csv.parse_with_header(text)
    rows |> NormalizeRows
}
```

This is already very Favnir-like:

- block is an expression
- `bind` introduces names only inside the block
- the last expression becomes the block value

### Recommendation

This should remain the default way to create short-lived local values.

It avoids leaking names into wider scope and keeps expression-oriented code natural.

## 3. `if`-Local Binding

There is also a useful second-stage feature:

```fav
if bind user <- find_user(id) {
    user.email
} else {
    "guest"
}
```

This is attractive, but it is more than syntax sugar.

It introduces semantic questions:

- what types are allowed on the right-hand side?
- does this mean `Option<T>` only?
- does it also work for `Result<T, E>`?
- does it imply success-pattern extraction?

### Recommendation

Do not make this a core v1 feature.

Instead:

- keep block-local `bind`
- keep `match` pattern binding
- treat `if bind ...` as a later ergonomic sugar candidate

That gives Favnir the useful parts now without forcing a special conditional binding semantics too early.

## 4. Binding Philosophy Summary

Favnir should follow this model:

- bindings never leak outside their lexical scope
- `match` bindings stay inside the selected arm
- block-local `bind` stays inside the block
- future `if bind` should also stay strictly local

This fits:

- immutable-only style
- lexical scope
- expression-oriented evaluation
- clear explain output

## 5. Synthetic Data Runes

Favnir and Veltra both benefit from being able to construct pipelines before real data exists.

This suggests an official Pure Favnir rune family for generated data.

Recommended structure:

- `random`
- `sample`

### `random`

Low-level deterministic random generation.

Typical responsibilities:

- `Random.int(min, max)`
- `Random.float(min, max)`
- `Random.bool()`
- `Random.choice(list)`
- `Random.string(len)`
- `Random.seed(n)`

This rune is for primitive and reproducible generation.

### `sample`

Higher-level structured synthetic data generation.

Typical responsibilities:

- `Sample.one<T>()`
- `Sample.list<T>(n)`
- `Sample.csv_rows<T>(n)`
- `Sample.json<T>()`
- domain helpers like `Sample.user()` or `Sample.order()` later

This rune is for pipeline development, testing, and notebook exploration.

## 6. Why `random` and `sample` Should Be Separate

They solve different problems.

`random`:

- primitive value generation
- seed control
- low-level reproducibility

`sample`:

- shape-aware records and lists
- data-source replacement
- onboarding and demos
- pipeline-first testing

Keeping them separate makes the API cleaner.

## 7. Type-Driven Sample Generation

One of the strongest possibilities is type-driven generation.

Example:

```fav
type User = {
    id: Int
    name: String
    email: String
}

bind user <- Sample.one<User>()
bind users <- Sample.list<User>(100)
```

This is valuable because Favnir already has explicit types.

It would make `sample` one of the first Pure Favnir libraries that clearly demonstrates the value of the type system outside the compiler itself.

## 8. Testing Value

This is particularly strong for tests.

Example:

```fav
test "aggregate users" {
    bind users <- Sample.list<User>(100, seed: 42)
    bind result <- AggregateUsers(users)
    assert(result.total == 100)
}
```

Benefits:

- easier pipeline tests
- less fixture maintenance
- reproducibility through seeds
- easier CI and notebook demos

## 9. Veltra Value

Veltra benefits directly from this feature family.

Notebook users often want to:

- prototype a flow before a real connector exists
- test a `flw` with representative fake rows
- inspect output shape early
- validate transforms without production data

`sample` makes that much easier.

This makes it more than a library convenience.
It becomes part of the product onboarding story.

## 10. Recommended Priority

Recommended implementation order:

1. keep `match` pattern binding as a core mechanism
2. keep block-local `bind` as the standard local-binding tool
3. postpone `if bind` until later
4. add `random` as a small pure rune
5. add `sample` on top of it as a higher-level pure rune

## 11. Summary

Recommended direction:

- local binding should stay lexical and non-leaking
- `match` and block-local binding are core
- `if bind` is a later ergonomic feature
- `random` and `sample` should be official rune families
- `sample` is especially valuable for tests, notebooks, and Veltra onboarding

These features make Favnir easier to use without making the language core heavier.
