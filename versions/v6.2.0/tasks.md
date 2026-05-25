# Favnir v6.2.0 Tasks

Date: 2026-05-25

## Goal
Verify full self-host bootstrap for the Favnir compiler.

Bootstrap target:

```text
Stage 1: Rust VM loads compiler.fav and compiles hello.fav -> bytecode_A
Stage 2: Rust VM loads compiler.fav and compiles compiler.fav -> compiler_artifact
Stage 3: Rust VM loads compiler_artifact and compiles hello.fav -> bytecode_B

Success condition: bytecode_A == bytecode_B
```

## Key work completed

- [x] Define and stabilize the self-host artifact format used by `compiler.fav`.
- [x] Implement Rust-side `FvcArtifact::from_bytes(...)` loading.
- [x] Repair self-host codegen so `compiler.fav` can compile itself.
- [x] Fix self-host match lowering issues:
  `next_local` propagation and nested variant pattern fail-path handling.
- [x] Remove fragile self-host reliance on multi-arg variant destructuring in hot codegen paths by switching to payload field access.
- [x] Keep `scan_collect`-based lexing in `self/compiler.fav` while teaching the Rust checker to allow direct `collect { helper(...) }` helpers that yield.
- [x] Verify `cargo test bootstrap_full_self_hosting -- --ignored --nocapture`.
- [x] Verify full `cargo test`.

## Validation snapshot

- [x] `cargo test self_hosted_compiler_type_checks -- --nocapture`
- [x] `cargo test bootstrap_full_self_hosting -- --ignored --nocapture`
- [x] `cargo test`

Observed final state:

- [x] Stage 1 succeeded
- [x] Stage 2 succeeded
- [x] Stage 3 succeeded
- [x] `bytecode_A == bytecode_B`
- [x] Test suite green: `1009 passed, 0 failed, 16 ignored`

## Remaining repo bookkeeping

- [x] Update `versions/v6.2.0/tasks.md`
- [x] Update `memory/MEMORY.md`
- [x] Commit only the v6.2.0 bootstrap completion changes
