# Favnir v1.0.0 Release Notes

Release date: 2026-05-01

## What's New

### Language Server Protocol (LSP)

- `fav lsp` starts an LSP server on stdin/stdout (JSON-RPC, Content-Length framing).
- Supports `textDocument/hover` — hover over any expression to see its inferred type.
- Supports `textDocument/publishDiagnostics` — parse and type errors appear as diagnostics in real time.
- VS Code usage: set `"favnir.lsp.command": ["fav", "lsp"]` in your extension settings.

### WASM: String Return Values (Phase 2)

- Functions returning `String` now compile to WASM using a `(ptr: i32, len: i32)` multi-value return.
- Example: `fn greet(name: String) -> String { name }` builds and runs correctly in WASM.
- `examples/string_wasm.fav` demonstrates String round-tripping via WASM.

### WASM: Closures (Phase 3)

- Closures can now be bound with `bind f <- |x| x + 1` and called as `f(5)` inside WASM programs.
- Captures of `Int`, `Float`, `Bool` values are stored in linear memory via the bump allocator.
- Under the hood: each closure generates a wrapper function in the WASM function table; calls use `call_indirect`.
- `examples/closures_wasm.fav` demonstrates no-capture and capturing closures.

### Rune Dependency Management (Phase 4)

- `fav.toml` now supports a `[dependencies]` section:
  ```toml
  [dependencies]
  mylib = { path = "../mylib" }
  utils = { registry = "local", version = "0.1.0" }
  ```
- `fav install` resolves all path and local-registry dependencies and writes `fav.lock`.
- `fav publish` validates `fav.toml` and prints instructions for local registry sharing.

## Known Limitations

- **WASM closures**: captures must be `Int`, `Float`, or `Bool`. `String`-capturing closures are not yet supported (returns W002).
- **WASM**: `List<T>` and `Map<V>` are not yet supported in WASM codegen (W001/W002).
- **WASM closures**: closures cannot be passed as arguments to other functions yet; only direct local call `f(args)` is supported.
- **LSP**: `textDocument/definition` returns `null` (go-to-definition not implemented).
- **LSP**: TCP mode (`fav lsp --port N`) is not yet implemented.
- **rune publish**: remote registry upload is not implemented; `fav publish` is local-only.

## Preview: v1.1.0

- `List<T>` and `Map<V>` in WASM via WasmGC or host-managed tables.
- LSP completion (`textDocument/completion`) for identifiers and field names.
- HTTP registry support for `fav install` and `fav publish`.
- Closure capture of `String` values in WASM.
