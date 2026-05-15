# Favnir v3.2.0 Migration Guide

## Summary

v3.2.0 has no breaking syntax changes relative to v3.1.0. Existing programs should continue to
compile without edits.

## New Features

### CSV rune

```favnir
import rune "csv"
```

Use:

- `csv.parse<T>(text)`
- `csv.parse_positional<T>(text)`
- `csv.write<T>(rows)`

### JSON rune

```favnir
import rune "json"
```

Use:

- `json.parse<T>(text)`
- `json.parse_list<T>(text)`
- `json.write<T>(value)`
- `json.write_list<T>(rows)`

### Positional CSV mapping

Use `#[col(n)]` on record fields when parsing headerless CSV:

```favnir
type Row = {
    #[col(0)] id: Int
    #[col(1)] name: String
}
```

### Schema conversion

`Schema.adapt` and `Schema.adapt_one` convert raw `Map<String, String>` rows into typed records.

## No Required Source Changes

- No keyword changes
- No parser-breaking syntax removals
- No mandatory migration step
