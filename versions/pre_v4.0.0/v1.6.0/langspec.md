# Favnir v1.6.0 Langspec

## Additions

### FString

Favnir v1.6.0 adds string interpolation with `$"..."`.

```fav
bind name <- "Favnir"
bind age <- 42
$"Hello {name}! Age: {age}"
```

Rules:
- Interpolation expressions are checked as normal expressions.
- `String`, `Int`, `Float`, and `Bool` are accepted directly.
- Other values require a `Show` implementation and are lowered through `Debug.show`.
- `\{` escapes a literal `{`.

Errors:
- `E053`: nested FString interpolation is not allowed.
- `E054`: interpolated value does not implement `Show`.

### Record Patterns

Favnir v1.6.0 supports record destructuring in patterns.

```fav
match user {
    { name, age } => name
    { name: n } => n
}
```

Supported forms:
- pun: `{ name, age }`
- alias: `{ name: n }`
- partial match: `{ name }`
- nested record patterns: `{ address: { city } }`

Errors:
- `E055`: record pattern used against a non-record value.
- `E056`: record pattern references an unknown field.

### assert_matches

Favnir v1.6.0 adds:

```fav
assert_matches(value, pattern)
```

This checks the given pattern at runtime and fails if the value does not match.

### fav test

`fav test` adds:
- `--filter <pattern>`
- improved PASS/FAIL summary output
- `assert_matches(...)`

### fav watch

`fav watch` adds a file watcher for `.fav` files.

```text
fav watch --cmd check
fav watch --cmd test
fav watch --cmd run src/main.fav
```
