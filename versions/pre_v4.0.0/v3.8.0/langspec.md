# Favnir v3.8.0 Language Specification

## New in v3.8.0: `grpc` rune + Protobuf CLI

`v3.8.0` adds the `Grpc` raw VM namespace, the `grpc` rune, `!Rpc`, `fav build --proto`, and `fav infer --proto`.

### Effects

- `Grpc.serve_raw` requires `!Rpc !Io`
- `Grpc.call_raw` requires `!Rpc`

### Builtins

- `Grpc.serve_raw(port: Int, service_name: String) -> Unit !Rpc !Io`
- `Grpc.call_raw(host: String, method: String, payload: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc`
- `Grpc.encode_raw(type_name: String, row: Map<String, String>) -> String`
- `Grpc.decode_raw(type_name: String, encoded: String) -> Map<String, String>`

### Types

```favnir
type RpcError = {
    code: Int
    message: String
}

type RpcRequest = {
    method: String
    payload: Map<String, String>
}
```

### `grpc` rune

- `grpc.serve`
- `grpc.call`
- `grpc.encode`
- `grpc.decode`
- `grpc.ok`
- `grpc.err`

### `fav build --proto`

Generates `proto3` schema from `type` record declarations and `interface` methods.

Mappings:

- `Int -> int64`
- `Float -> double`
- `String -> string`
- `Bool -> bool`
- `Option<T> -> optional T`
- `List<T> -> repeated T`
- `Result<T, E> -> T`
- `Stream<T> -> stream T`
- `Unit -> google.protobuf.Empty`

### `fav infer --proto`

Infers Favnir `type` and `interface` declarations from a `.proto` file.
