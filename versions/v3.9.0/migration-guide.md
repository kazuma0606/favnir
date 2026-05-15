# v3.9.0 Migration Guide

No source-level breaking language changes.

Runtime changes:

- `grpc.serve` and `grpc.call` now target gRPC-style framed transport.
- `grpc.serve_stream` and `grpc.call_stream` are new in `v3.9.0`.
- `fav build --proto` continues to emit `stream` responses for `Stream<T>` interface methods.
