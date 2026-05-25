# MEMORY

## v6.2.0

- Verified full self-host bootstrap for the Favnir compiler.
- `compiler.fav` now compiles `compiler.fav` into a loadable artifact, and that artifact recompiles `hello.fav` to identical bytecode.
- Rust checker now permits direct `collect { helper(...) }` helper functions that contain `yield`, without relaxing the general `yield outside collect` error.
- Validation passed with `cargo test bootstrap_full_self_hosting -- --ignored --nocapture` and full `cargo test` (`1009 passed, 0 failed, 16 ignored`).

## v3.9.0

- Upgraded gRPC runtime internals toward framed transport.
- Added `Grpc.serve_stream_raw` and `Grpc.call_stream_raw`.
- Added `grpc.serve_stream` and `grpc.call_stream`.
