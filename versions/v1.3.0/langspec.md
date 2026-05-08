# Favnir v1.3.0 Langspec

This version adds abstract pipeline building blocks:

- `abstract trf`
- `abstract flw`
- bound `flw` templates

## `abstract trf`

An abstract transform declares a callable contract without a body.

```favnir
abstract trf FetchUser: UserId -> User? !Db
```

Rules:

- it has a name, input type, output type, and optional effects
- it cannot be called directly at runtime
- direct calls fail in the checker with `E051`

## `abstract flw`

An abstract flow is a reusable template made from named slots.

```favnir
abstract flw DataPipeline<Row> {
    parse: String -> List<Row>!
    validate: Row -> Row!
    save: List<Row> -> Int !Db
}
```

Each slot has:

- a slot name
- an input type
- an output type
- optional effects

Type parameters on the template are substituted into slot types during binding.

## Bound `flw`

A concrete flow binds implementations into the template slots.

```favnir
flw UserImport = DataPipeline<UserRow> {
    parse <- ParseCsv
    validate <- ValidateUser
    save <- SaveUsers
}
```

Rules:

- unknown slots produce `E049`
- mismatched slot signatures produce `E048`
- fully bound templates lower to executable flow functions
- partially bound templates become `PartialFlw`

## Partial Binding

If one or more slots are left unbound, Favnir keeps the binding as a partial flow description.

```favnir
flw PartialImport = DataPipeline<UserRow> {
    parse <- ParseCsv
    save <- SaveUsers
}
```

This value:

- may appear in `fav explain`
- emits `W011` in `fav check`
- fails with `E050` in `fav run` and `fav build`

## Effect Rule

The effect of a fully bound abstract flow is the union of all slot effects in template order.

## Explain Output

`fav explain` shows:

- `ABSTRACT TRF`
- `ABSTRACT FLW`
- `FLW BINDINGS`

For bound flows, explain includes:

- template name
- slot-to-implementation bindings
- whether the binding is `complete` or `partial`
- combined effects for complete bindings

## Error Codes

- `E048`: slot signature mismatch
- `E049`: unknown slot name in bound flow
- `E050`: partial flow used in run/build
- `E051`: direct call to `abstract trf`

## Examples

- `examples/abstract_flw_basic.fav`
- `examples/abstract_flw_inject.fav`
