# Favnir v6.2.0 Semantic Gap Audit

Date: 2026-05-25

## Scope

This note records the current semantic and ownership gaps between:

- the Rust-hosted Favnir implementation
- the self-host compiler in `fav/self/compiler.fav`

The goal is not to eliminate every difference in v6.2.0.
The goal is to identify which differences:

- must be reduced for stronger self-host authority
- are acceptable bootstrap-support exceptions for now
- should remain in Rust as part of the trusted kernel

## Current level summary

Favnir is currently `Level 3 / 5`:
`Bootstrap Verified`.

That means:

- the self-host compiler can compile itself
- the compiled artifact can recompile the same workload
- the verified bootstrap path produces matching output

But it does not yet mean that the self-host compiler is the primary semantic authority across the whole pipeline.

## Gap inventory

### 1. `collect { helper(...) }` is a Rust-side bootstrap exception

Status:

- Rust checker normally rejects `yield` outside a direct `collect` context.
- v6.2.0 adds a Rust-side helper-discovery pass so functions directly invoked from `collect { helper(...) }` are treated as collect-capable.
- This exists to keep `self/compiler.fav` using `scan_collect(...)` without forcing a more expensive structural rewrite.

Evidence:

- `fav/src/middle/checker.rs`
  `register_collect_helpers(...)`
  `collect_helpers_in_expr(...)`
  `self.collect_helpers.contains(&fd.name)`
- `fav/self/compiler.fav`
  `scan_collect(chars: List<String>) -> Bool`

Classification:

- bootstrap-support exception
- self-host candidate

Reason:

This is not a language-level principle that should be permanently owned by Rust.
It is a compatibility bridge to keep the current self-host lexer shape alive.

Recommended direction:

- either remove the helper pattern from `compiler.fav`
- or define helper-enabled collect semantics explicitly at the language level and implement them consistently on both sides

### 2. Self-host AST and lowering are intentionally smaller and more ad hoc than the Rust AST

Status:

- Rust uses the canonical frontend AST in `fav/src/ast.rs`.
- `compiler.fav` uses a compact self-host AST with constructors such as `ECollect`, `EYield`, `EArm`, `EArgList`, `EField`, and record-style payload encoding.
- The self-host AST is sufficient for bootstrap, but it is not yet a principled mirror of the Rust-side language model.

Evidence:

- `fav/src/ast.rs`
  `Expr::Collect`, `Stmt::Yield`
- `fav/self/compiler.fav`
  `ECollect(Expr)`, `EYield(Expr)`, `EArgList(...)`, `EField(...)`

Classification:

- intentionally shared behavior at the language level
- self-host candidate at the representation level

Reason:

The semantic feature is shared.
The representation is not.
That is acceptable for now, but it increases the chance of future divergence.

Recommended direction:

- keep representation differences if they help bootstrap simplicity
- add focused regression tests whenever a Rust AST feature is encoded differently in self-host

### 3. Match/pattern lowering still depends on self-host-specific codegen structure

Status:

- v6.2.0 fixed important self-host match/codegen issues:
  `next_local` propagation and nested variant-pattern fail-path composition.
- These fixes were local to `compiler.fav` codegen rather than produced from a shared lowering contract.

Evidence:

- `fav/self/compiler.fav`
  `compile_match_dispatch_finish(...)`
  `compile_match_arms(...)`

Classification:

- self-host candidate

Reason:

This is compiler-pipeline behavior that should trend toward self-host ownership.
Rust should validate it with tests, but should not remain the only place where the semantics are dependable.

Recommended direction:

- document the expected match lowering contract
- add regression cases for nested variant patterns and local-slot propagation

### 4. Call argument lowering in self-host currently relies on record payload field access conventions

Status:

- v6.2.0 removed fragile dependence on multi-arg variant destructuring in hot self-host codegen paths.
- The replacement is payload record access via fields such as `parts._0`, `parts._1`, `parts._2`.

Evidence:

- `fav/self/compiler.fav`
  `count_args(...)`
  `compile_args(...)`
  `compile_expr(...)` cases for `EBinOp`, `EIf`, `EBind`, `ECall`, `EAccess`, `EMatch`, `ELambda`, `ERecordLit`

Classification:

- self-host candidate

Reason:

This is a pragmatic encoding choice, not a Rust-kernel concern.
It should be stabilized by self-host tests and documentation, not by ad hoc knowledge.

Recommended direction:

- document payload field ordering as a self-host internal contract
- add regression tests for nested calls and pattern-heavy lowering paths

### 5. Rust remains the semantic authority for type-check acceptance

Status:

- `self_hosted_compiler_type_checks` still evaluates `self/compiler.fav` with the Rust checker.
- This is currently the main gate that decides whether the self-host source is acceptable.

Evidence:

- `fav/src/driver.rs`
  `self_hosted_compiler_type_checks()`

Classification:

- intentionally shared behavior today
- self-host candidate over time

Reason:

For v6.2.0 this is the right safety gate.
But long term, a strong self-host system should not require Rust to be the only semantic judge of compiler source validity.

Recommended direction:

- keep this Rust gate
- add more cases where self-host semantics are exercised and compared explicitly

### 6. Bootstrap truth is still defined by Rust-side execution and artifact loading

Status:

- the decisive bootstrap proof is executed by Rust:
  source loading, artifact loading, VM execution, and bytecode comparison.

Evidence:

- `fav/src/driver.rs`
  `bootstrap_full_self_hosting()`
- Rust-side `FvcArtifact` loading and VM execution path

Classification:

- Rust-kernel candidate

Reason:

This is a healthy place to keep Rust for now.
Artifact loading, VM execution, and low-level binary/runtime trust boundaries are appropriate kernel responsibilities.

Recommended direction:

- keep Rust responsible here in v6.2.0
- document the boundary explicitly rather than treating it as an accidental dependency

### 7. Some backend targets do not treat `collect`/`yield` as first-class supported codegen paths

Status:

- Rust IR/codegen understands `Collect` and `Yield`
- some alternative backends still treat them as unsupported or partial

Evidence:

- `fav/src/backend/wasm_codegen.rs`
  unsupported paths for `IRExpr::Collect` and `IRStmt::Yield`

Classification:

- intentionally shared behavior
- Rust-kernel candidate for non-bootstrap targets

Reason:

This does not block the current self-host bootstrap.
It matters for broader language/backend parity, not for the narrow trusted kernel.

Recommended direction:

- keep separate from core bootstrap work
- track as backend parity, not as a blocker for self-host authority

## Bootstrap-support exceptions currently in force

The current explicit bootstrap-support exceptions are:

1. Rust checker accepts direct `collect { helper(...) }` helper functions that contain `yield`.
2. `self/compiler.fav` retains a helper-style lexer structure instead of moving `yield` directly into a flatter collect block.
3. Self-host internal AST/payload encoding remains intentionally compact and non-canonical as long as bootstrap stays verified.

These exceptions are acceptable in v6.2.0, but they should be treated as named exceptions, not invisible defaults.

## Classification summary

### Self-host candidates

- collect helper semantics, if kept as a real feature
- match/pattern lowering contract
- call argument / payload field lowering contract
- long-term ownership of compiler-source type semantics

### Rust-kernel candidates

- artifact loading
- VM execution
- low-level binary/runtime trust boundaries
- security-sensitive and memory-sensitive infrastructure

### Intentionally shared behavior

- surface language features such as `collect`, `yield`, pattern matching, and blocks
- validation gates that compare Rust-hosted behavior against self-host behavior

## Recommended next move

The best first follow-up is:

1. choose one semantic area
2. document its expected behavior
3. add a focused regression test
4. make Rust and self-host converge there intentionally

Recommended first target:

- block / collect / yield semantics

Reason:

That area already contains an explicit bootstrap exception, so it offers the highest leverage for moving from "verified" toward "owned".
