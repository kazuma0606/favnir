# MEMORY

## v6.2.0

- Verified full self-host bootstrap for the Favnir compiler.
- `compiler.fav` now compiles `compiler.fav` into a loadable artifact, and that artifact recompiles `hello.fav` to identical bytecode.
- Rust checker now permits direct `collect { helper(...) }` helper functions that contain `yield`, without relaxing the general `yield outside collect` error.
- Validation passed with `cargo test bootstrap_full_self_hosting -- --ignored --nocapture` and full `cargo test` (`1009 passed, 0 failed, 16 ignored`).
- Added v6.2.0 self-host authority notes: `self_host_maturity.md`, `semantic_gap_audit.md`, `bootstrap_contract.md`, and `self_host_ast_contract.md`.
- Fixed and regression-covered three bootstrap-sensitive semantic areas: `block / collect / yield`, `pattern / match`, and `call argument lowering / record payload access`.
- Extended bootstrap comparison beyond `hello.fav` with a `match` / `collect` / record-heavy source shape.
- Kept Rust explicitly as the trusted kernel for artifact loading, VM execution, binary/runtime boundaries, and safety-sensitive infrastructure.

## v3.9.0

- Upgraded gRPC runtime internals toward framed transport.
- Added `Grpc.serve_stream_raw` and `Grpc.call_stream_raw`.
- Added `grpc.serve_stream` and `grpc.call_stream`.
