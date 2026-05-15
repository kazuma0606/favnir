# Favnir v3.2.0 Language Specification

## Overview

Favnir v3.2.0 adds first-party `csv` and `json` runes built on top of new raw CSV/JSON VM
primitives and schema adaptation via `Schema.adapt`.

## New Runtime Types

### `SchemaError`

```favnir
type SchemaError = {
    field: String
    expected: String
    got: String
}
```

`SchemaError` is the standard error payload used by v3.2.0 CSV/JSON schema conversion APIs.

### `CsvOptions`

```favnir
type CsvOptions = {
    delimiter: String
    has_header: Bool
}
```

`CsvOptions` is defined by the `csv` rune and is used by `csv.parse_with_opts<T>`.

## Field Attributes

Record fields may declare attributes in the form `#[name(arg)]`.

v3.2.0 introduces:

```favnir
type Row = {
    #[col(0)] id: Int
    #[col(1)] name: String
}
```

Rules:

- `#[col(n)]` maps a field to a positional CSV column.
- `n` must be a non-negative integer.
- Attributes are stored in the AST as `FieldAttr { name, arg }`.

## New Builtins

### CSV raw builtins

```favnir
Csv.parse_raw(text: String, delimiter: String, has_header: Bool) -> Result<List<Map<String, String>>, SchemaError>
Csv.write_raw(rows: List<Map<String, String>>, delimiter: String) -> String
```

`Csv.parse_raw` returns header-based maps when `has_header = true`, and `"0"`, `"1"`, ... keys when
`has_header = false`.

### JSON raw builtins

```favnir
Json.parse_raw(text: String) -> Result<Map<String, String>, SchemaError>
Json.parse_array_raw(text: String) -> Result<List<Map<String, String>>, SchemaError>
Json.write_raw(map: Map<String, String>) -> String
Json.write_array_raw(rows: List<Map<String, String>>) -> String
```

v3.2.0 supports only flat object/array conversion. Nested objects and arrays are out of scope.

### Schema builtins

```favnir
Schema.adapt(rows: List<Map<String, String>>, type_name: String) -> Result<List<T>, SchemaError>
Schema.adapt_one(row: Map<String, String>, type_name: String) -> Result<T, SchemaError>
Schema.to_csv(rows: List<T>, type_name: String) -> String
Schema.to_json(value: T, type_name: String) -> String
Schema.to_json_array(rows: List<T>, type_name: String) -> String
```

Supported field conversions:

- `Int`
- `Float`
- `Bool`
- `String`
- `Option<T>`

`Option<T>` treats the empty string as `None`.

## `type_name_of<T>()`

`type_name_of<T>()` is a compile-time helper that lowers to a string literal of the concrete type
name and is intended for `Schema.*` builtins and generic rune wrappers.

## Official Runes

### `csv`

- `csv.parse<T>(text)`
- `csv.parse_positional<T>(text)`
- `csv.parse_with_opts<T>(text)(opts)`
- `csv.write<T>(rows)`

### `json`

- `json.parse<T>(text)`
- `json.parse_list<T>(text)`
- `json.write<T>(value)`
- `json.write_list<T>(rows)`

## Error Codes

- `E0501`: schema field missing
- `E0502`: schema type mismatch
- `E0503`: invalid `#[col(n)]` index
- `E0504`: json parse error
- `E0505`: csv parse error

## Version Summary

- New syntax: `#[col(n)]`
- New helper: `type_name_of<T>()`
- New VM builtins: `Csv.parse_raw`, `Csv.write_raw`, `Json.parse_raw`, `Json.parse_array_raw`,
  `Json.write_raw`, `Json.write_array_raw`, `Schema.adapt`, `Schema.adapt_one`,
  `Schema.to_csv`, `Schema.to_json`, `Schema.to_json_array`
- New official runes: `csv`, `json`
