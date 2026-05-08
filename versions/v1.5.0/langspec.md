# Favnir v1.5.0 Langspec

## Additions

- `fav explain diff <from> <to> [--format text|json]`
- `fav graph --focus fn [--entry <name>] [--depth <n>]`
- top-level `effect Name` declarations
- lint rules `L005`, `L006`, `L007`

## Custom Effects

Custom effects are declared at top level.

```fav
public effect Payment
effect Notification
```

Declared effects may be used anywhere effect annotations are accepted.

```fav
trf Charge: Int -> Int !Payment = |x| { x }
fn notify() -> Unit !Notification { () }
```

Using an undeclared custom effect is a checker error:

- `E052`: undeclared effect

Built-in effects remain always available:

- `Io`
- `Db`
- `Network`
- `File`
- `Trace`
- `Emit<T>`

## Explain Diff

`fav explain diff` compares two explain payloads generated from:

- `.fav`
- `.json`
- `.fvc`

Text output highlights:

- added entries with `+`
- removed entries with `-`
- changed entries with `~`
- breaking changes summary

JSON output includes:

- `from_label`
- `to_label`
- `fn_changes`
- `trf_changes`
- `flw_changes`
- `type_changes`
- `effects_added`
- `effects_removed`
- `breaking_changes`

## Function Graph Focus

`fav graph --focus fn` renders a function dependency graph.

Options:

- `--entry <name>`: start traversal from a specific entry point
- `--depth <n>`: limit traversal depth
- `--format text|mermaid`

## Lint Rules

- `L005`: unused private `trf` / `abstract trf` / `flw` / `abstract flw` / `flw binding`
- `L006`: `trf` name is not PascalCase
- `L007`: `effect` name is not PascalCase
