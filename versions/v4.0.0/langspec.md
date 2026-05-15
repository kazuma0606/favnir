# Favnir v4.0.0 Language Specification

## Summary

v4.0.0 is the gRPC hardening + backlog-clearing release. The language surface is
unchanged from v3.x; the changes are entirely in the runtime and toolchain.

---

## gRPC (real HTTP/2)

### `Grpc.serve_raw(port: Int, service_name: String) -> Unit !Io !Rpc`

Binds an HTTP/2 gRPC server on `port` and enters an infinite dispatch loop.
For each incoming request the VM resolves a handler function named
`handle_<method_snake>` (e.g. `GetUser` → `handle_get_user`) and calls it with
the decoded `Map<String,String>` payload.  The function never returns.

**Handler signature:**
```
public fn handle_<method>(req: Map<String, String>) -> Map<String, String> { … }
```

### `Grpc.serve_stream_raw(port: Int, service_name: String) -> Unit !Io !Rpc`

Same as `serve_raw` but the handler returns `List<Map<String, String>>`.  Each
element is encoded as a separate gRPC data frame in the response body.

### `Grpc.call_raw(host: String, method: String, payload: Map<String,String>) -> Result<Map<String,String>, RpcError> !Rpc`

Makes a real HTTP/2 unary gRPC call.  On success returns `Ok(response_map)`.
On connection failure returns `Err(RpcError { code: 14, … })`.
Checks `grpc-status` trailer; non-zero status maps to `Err`.

### `Grpc.call_stream_raw(host: String, method: String, payload: Map<String,String>) -> List<Map<String,String>> !Rpc`

Makes a real HTTP/2 server-streaming gRPC call.  Returns all response frames as
a list.  On connection failure returns an empty list.

---

## `pipe match` (`|> match { … }`) — existing since v3.x

```
expr |> match {
    Ok(v)  => v
    Err(_) => default
}
```

Desugars to a normal `match` at parse time; no new AST node.

---

## Pattern guards (`where`) — existing since v3.x

```
match score {
    n where n >= 90 => "A"
    n where n >= 70 => "B"
    _               => "C"
}
```

Guard expression must be `Bool`; type-checked as E027 if not.

---

## Stack traces — existing since v3.x

Runtime errors include a stack trace:
```
RuntimeError: division by zero
  at divide (src/main.fav:12)
  at main (src/main.fav:20)
```

---

## Dependencies added in v4.0.0

| Crate   | Version | Purpose                          |
|---------|---------|----------------------------------|
| h2      | 0.3     | HTTP/2 frame-level gRPC transport|
| bytes   | 1       | Zero-copy byte buffers for h2    |
| http    | 0.2     | HTTP types (Request/Response)    |
