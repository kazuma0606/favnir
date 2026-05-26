# Favnir v6.2.0 Bootstrap Contract

Date: 2026-05-25

## Purpose

This note documents the contract that currently matters for self-host bootstrap between:

- `fav/self/compiler.fav`
- the Rust artifact loader and VM

This is not a full language spec.
It is the narrower contract that must hold for:

- self-host compilation
- artifact loading
- bootstrap replay
- `bytecode_A == bytecode_B`

## Contract layers

There are three distinct layers:

1. Language-level contract
2. IR/lowering contract
3. Artifact/VM contract

These should not be conflated.

## 1. Language-level contract

The following are language-level features required by the bootstrap path:

- records and record field access
- variant constructors and pattern matching
- nested variant patterns
- blocks and local binds
- `collect { ... }` and `yield`
- multi-argument calls
- closures used by parts of the self-host pipeline

These features are part of the Favnir language surface.
Their exact internal representation is not part of the language contract.

## 2. IR/lowering contract

Rust IR currently represents the relevant bootstrap features using:

- `IRExpr::Call`
- `IRExpr::FieldAccess`
- `IRExpr::Collect`
- `IRExpr::RecordConstruct`
- `IRStmt::Yield`

Evidence:

- `fav/src/middle/ir.rs`

This means the bootstrap path depends on lowering these source constructs into stable IR shapes before backend codegen.

Important distinction:

- the existence of call / field / collect / record constructs in IR is contract-relevant
- the exact self-host AST encoding used inside `compiler.fav` is not contract-relevant by itself

## 3. Artifact / VM contract

### Artifact file identity

The self-host serializer emits an artifact with:

- magic: `FVC\x01`
- version bytes: `[0x20, 0, 0, 0]`

Evidence:

- `fav/self/compiler.fav`
  `serialize_artifact(...)`
- `fav/src/backend/artifact.rs`
  `FvcArtifact::from_bytes(...)`

If these differ, Rust cannot load the self-hosted output.

### Artifact contents

At minimum, bootstrap depends on these sections being mutually understood:

- string table
- function table
- globals table
- per-function constants
- per-function bytecode

The self-host serializer and Rust loader must agree on:

- field order
- integer widths
- little-endian encoding
- constant tagging

Evidence:

- `fav/self/compiler.fav`
  `write_u32_le`, `write_i64_le`, `serialize_artifact(...)`
- `fav/src/backend/artifact.rs`
  `FvcArtifact::from_bytes(...)`

## Bootstrap-relevant opcodes

The bootstrap path specifically relies on these opcode meanings matching across self-host and Rust:

- `CallNamed`
- `JumpIfNotVariantC`
- `MakeClosureN`
- `GetField`
- `BuildRecord`
- `CollectBegin`
- `CollectEnd`
- `YieldValue`

Evidence:

- Rust opcode definitions:
  `fav/src/backend/codegen.rs`
- Rust VM execution:
  `fav/src/backend/vm.rs`
- self-host opcode mapping:
  `fav/self/compiler.fav`
  `opcode_byte(...)`

Current byte assignments used by bootstrap:

- `GetField = 0x40`
- `BuildRecord = 0x41`
- `CollectBegin = 0x50`
- `CollectEnd = 0x51`
- `YieldValue = 0x52`
- `CallNamed = 0x56`
- `JumpIfNotVariantC = 0x57`
- `MakeClosureN = 0x5A`

Note: `GetFieldC (0x58)` and `BuildRecordC (0x59)` exist in the Rust VM but are not
currently emitted by `compiler.fav` and are therefore not bootstrap-contract-relevant.

These byte values are implementation-level, but they are contract-relevant as long as Rust VM directly executes the produced artifact bytes.

## Self-host internal representations that affect bootstrap

The following self-host details are not language-level contracts, but they do affect bootstrap correctness:

- `EArgList` ordering
- `EField` ordering
- record-style payload access through `parts._0`, `parts._1`, `parts._2`
- reversed accumulation and restoration of match arms
- nested variant pattern lowering and fail-path assembly

Evidence:

- `fav/self/compiler.fav`
  `count_args(...)`
  `compile_args(...)`
  `reverse_match_arms(...)`
  `compile_match_dispatch_finish(...)`
  `compile_match_arms(...)`

These are self-host implementation details.
They are not the language contract.
But they must stay stable enough for bootstrap to remain reproducible.

## What is contract-level vs implementation-level

### Language-level contract

- records exist
- fields can be accessed
- variants can be matched
- nested patterns are legal
- `collect` and `yield` exist
- multi-argument calls preserve argument order

### IR/lowering contract

- records lower to record-construction and field-access shapes
- collect/yield lower to collect-expression plus yield-statements
- calls lower to explicit callee plus ordered argument list
- match lowers in a way that preserves fallthrough and local bindings

### Artifact/VM contract

- `FVC\x01` artifact identity
- section ordering and numeric encoding
- constant tagging
- opcode byte assignments
- VM behavior for the bootstrap-relevant instructions

### Implementation details

- exact self-host AST constructors
- exact accumulator shapes in the self-host parser
- exact helper-function decomposition such as `scan_collect(...)`
- whether a given lowering routine is split into one or several helper functions

## Current trusted kernel boundary

For v6.2.0, the following should be treated as Rust-side trusted-kernel responsibilities:

- artifact byte loading
- opcode dispatch and VM execution
- low-level binary decoding rules
- low-level runtime stack/value behavior
- security-sensitive and memory-sensitive substrate

This is intentional.
It is not a sign that bootstrap is incomplete.

## Current bootstrap-sensitive exceptions

One explicit semantic bridge exists today:

- Rust checker recognizes direct `collect { helper(...) }` helper functions that contain `yield`

That exception is bootstrap-sensitive because `self/compiler.fav` currently uses helper-based collect scanning in the lexer.

This should be treated as a named compatibility rule, not an invisible default.

## Working conclusion

To preserve bootstrap, the highest-priority shared contract is:

- artifact layout
- bootstrap-relevant opcode meaning
- lowering behavior for records, calls, collect/yield, and nested variant matches

Everything else should be treated either as:

- a language surface contract to keep stable
- or an implementation detail that may change as long as the bootstrap contract remains intact
