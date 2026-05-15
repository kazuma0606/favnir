# Favnir v3.x → v4.0.0 Migration Guide

## Breaking changes

### `Grpc.serve_raw` / `grpc.serve` now blocks forever

In v3.9.0 these calls returned immediately (the server was a stub that did
nothing).  In v4.0.0 they enter a real dispatch loop and **never return**.

**Before (v3.x):**
```
public fn main() -> Unit !Io !Rpc {
    grpc.serve(50051, "UserService")
    IO.println("this line was reachable in v3.x") // unreachable now
}
```

**After (v4.0.0):**
```
public fn main() -> Unit !Io !Rpc {
    IO.println("server starting on :50051")
    grpc.serve(50051, "UserService")   // blocks here forever
}
```

Any code after the `serve` call is dead code in v4.0.0.

### Handler functions must accept `Map<String, String>`

In v3.x `handle_*` functions accepted `RpcRequest`.
In v4.0.0 they must accept `Map<String, String>` (the decoded proto fields).

**Before (v3.x):**
```
public fn handle_get_user(req: RpcRequest) -> Result<Map<String, String>, RpcError> {
    bind id <- Option.unwrap_or(Map.get(req.payload, "id"), "0")
    …
}
```

**After (v4.0.0):**
```
public fn handle_get_user(req: Map<String, String>) -> Map<String, String> {
    bind id <- Option.unwrap_or(Map.get(req, "field1"), "0")
    …
}
```

Note: proto field names are positional (`field1`, `field2`, …) because the
proto wire format does not carry field names.

### `Grpc.call_raw` no longer returns code 12

In v3.9.0 a successful TCP connection still returned
`Err(RpcError { code: 12, … })`.  In v4.0.0 it performs a real HTTP/2 exchange
and returns `Ok(response_map)` on success.

## New dependencies

Add these to your `Cargo.toml` if you build the Favnir runtime from source:
```toml
h2    = "0.3"
bytes = "1"
http  = "0.2"
```
