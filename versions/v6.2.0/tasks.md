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

## Additional v6.2.0 tasks

Goal for these additions:
move Favnir from "bootstrap verified" toward "self-hosted authority"
without forcing unsafe or specialist-heavy runtime work out of Rust.

### A. Semantic gap audit

- [x] A-1: Produce a gap memo for Rust checker vs `compiler.fav` semantics.
- [x] A-2: List every current bootstrap-support exception, including `collect { helper(...) }` handling.
- [x] A-3: Classify each gap as:
  self-host candidate, Rust-kernel candidate, or intentionally shared behavior.

### B. Self-host authority expansion

- [x] B-1: Pick 1 parser/checker semantic area that can move toward self-host ownership without touching the VM.
- [x] B-2: Add focused tests that fail when Rust and self-host diverge for that area.
- [x] B-3: Implement the selected semantic alignment in `compiler.fav` and/or supporting Rust glue.

Recommended first targets:

- pattern and match behavior
- block / collect / yield semantics
- call argument lowering and record payload access

Selected first target:

- [x] block / collect / yield semantics

Additional aligned target completed in v6.2.0:

- [x] pattern / match behavior
  nested variant pattern fallback and arm-local bindings are now covered by focused regression tests
- [x] call argument lowering and record payload access
  multi-argument calls using record field access are now covered by focused checker and runtime regressions

### C. Trusted-kernel boundary

- [x] C-1: Write a short note defining the Rust trusted kernel for Favnir.
- [x] C-2: Explicitly keep the following areas in Rust unless specialist review exists:
  cryptography, security-sensitive primitives, low-level binary boundaries, network protocol robustness, memory-sensitive runtime internals.
- [x] C-3: Mark non-goals for v6.2.0 so "self-host progress" is not confused with "rewrite everything in Favnir".

### D. Contract documentation

- [x] D-1: Document the artifact format contract used between `compiler.fav` and the Rust VM.
- [x] D-2: Document the opcode / IR assumptions required by bootstrap.
- [x] D-3: Identify which parts are language-level contracts vs implementation details.

### E. Validation hardening

- [x] E-1: Keep `self_hosted_compiler_type_checks` as a mandatory gate for self-host changes.
- [x] E-2: Keep `bootstrap_full_self_hosting` as a mandatory gate for compiler pipeline changes.
- [x] E-3: Add at least one regression test for a previously mismatched Rust/self-host semantic edge.

### F. Post-bootstrap hardening

Goal for this section:
reduce the remaining "borrowed" bootstrap behavior and strengthen self-host confidence within v6.2.0.

- [x] F-1: Reduce or eliminate the current `collect { helper(...) }` bootstrap exception if feasible within v6.2.0.
- [x] F-2: Expand parser/checker regression coverage for remaining self-host-sensitive semantic edges.
  Added nested variant guard and broader self-host-sensitive runtime regressions.
- [x] F-3: Add a short contract note for the self-host internal AST encodings used by `compiler.fav`.
  Recorded in `self_host_ast_contract.md`.
- [x] F-4: Strengthen bootstrap comparison beyond the current primary `hello.fav` path with one or more additional source shapes.
  Added `bootstrap_full_self_hosting_on_match_collect_source`.

Planned execution order:

1. `collect` helper dependency review and reduction
2. semantic edge regression expansion
3. self-host AST contract note
4. broader bootstrap comparison inputs

### G. Remaining self-host authority follow-ups

Goal for this section:
increase self-host confidence within v6.2.0 without widening the Rust trusted kernel
or forcing a risky compiler rewrite late in the version.

- [x] G-1: Add negative regression coverage for the narrowed `collect { helper(...) }` exception.
  Confirmed the exception does not extend through `trf`, `test`, `bench`, indirect calls, or non-tail helper usage.
- [x] G-2: Add one more bootstrap comparison input that stresses closure capture and `for`-inside-`collect`.
  Added `bootstrap_full_self_hosting_on_closure_collect_source` and a faster self-host artifact regression for closure capture + `for`-inside-`collect` execution.
- [x] G-3: Expand focused self-host-sensitive regressions around lowering helpers.
  Target areas such as `free_names`, `collect_capture_names`, `compile_args`, and `compile_match_arms`.
  Progress so far: added closure-capture + `for`-inside-`collect` self-host regressions, nested-variant later-arm and guarded-arm regressions, a multi-capture closure regression that exercises capture selection with unused locals mixed in, a guarded-match bootstrap comparison, and a nested-call regression that mixes closure bodies with record-field arguments.

Recommended execution order:

1. negative `collect` helper boundary regressions
2. closure/capture bootstrap comparison input
3. additional lowering regressions
