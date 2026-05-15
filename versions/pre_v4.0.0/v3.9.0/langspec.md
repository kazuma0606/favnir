# Favnir v3.9.0 Language Specification

## New in v3.9.0: gRPC framing + streaming helpers

`v3.9.0` keeps the `Grpc` and `grpc` APIs from `v3.8.0`, but updates the runtime contract to use gRPC-style 5-byte framing and adds raw streaming helpers.

### Builtins

- `Grpc.serve_raw(port: Int, service_name: String) -> Unit !Rpc !Io`
- `Grpc.call_raw(host: String, method: String, payload: Map<String, String>) -> Result<Map<String, String>, RpcError> !Rpc`
- `Grpc.serve_stream_raw(port: Int, service_name: String) -> Unit !Rpc !Io`
- `Grpc.call_stream_raw(host: String, method: String, payload: Map<String, String>) -> List<Map<String, String>> !Rpc`
- `Grpc.encode_raw(type_name: String, row: Map<String, String>) -> String`
- `Grpc.decode_raw(type_name: String, encoded: String) -> Map<String, String>`

### Frame Format

- byte `0`: compression flag, currently always `0`
- bytes `1..4`: payload length as big-endian `u32`
- bytes `5..`: protobuf payload bytes

### `grpc` rune

- `grpc.serve`
- `grpc.call`
- `grpc.serve_stream`
- `grpc.call_stream`
- `grpc.encode`
- `grpc.decode`
- `grpc.ok`
- `grpc.err`

### Streaming

`grpc.call_stream` and `Grpc.call_stream_raw` return `List<Map<String, String>>`. Current VM integration decodes one or more framed payloads and flattens them into a list of records.
