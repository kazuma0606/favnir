# Favnir v6.2.0 Self-Host AST Contract

Date: 2026-05-25

## Purpose

This note captures the internal AST encodings inside `fav/self/compiler.fav`
that are currently bootstrap-sensitive.

This is not the public language spec.
It is the narrower contract needed so that:

- the Rust-hosted compiler can compile `compiler.fav`
- the self-hosted compiler can compile itself and representative user programs
- Stage 1 and Stage 3 keep producing identical bytecode

## Scope

This note covers:

- `Expr`
- `Pat`
- AST list encodings used by calls, records, and match arms

It does not define:

- the artifact file format
- the VM opcode format
- Rust IR shapes

Those are documented separately in `bootstrap_contract.md`.

## Contract summary

The self-host compiler currently depends on these internal shapes:

- `ECall(ns, fname, args)`
- `EAccess(obj, field)`
- `ERecordLit(tname, fields)`
- `EField(fname, val_e, rest)`
- `EMatch(scrut, arms)`
- `EArm(pat, body, rest)`
- `EArgList(head, tail)`
- `PVariant(name)`
- `PVariantP(name, inner)`

These are internal encodings, but several of them are bootstrap-sensitive because
they are consumed directly by `compile_expr`, `compile_pattern_dispatch`,
`compile_record_fields`, `count_args`, and related helpers.

## Multi-arg payload rule

For v6.2.0, the important operational rule is:

- treat multi-argument variant payloads as payload records in bootstrap-sensitive code

In practice, this means code such as:

- `EField(parts)` with `parts._0`, `parts._1`, `parts._2`
- `EArm(parts)` with `parts._0`, `parts._1`, `parts._2`
- `PVariantP(parts)` with `parts._0`, `parts._1`

This rule matters in self-host codegen and analysis helpers.
Using direct multi-argument destructuring in those paths can still pass some tests,
but it is not reliable under self-compilation.

## List encodings

Several AST fragments are represented as right-linked internal lists:

- call arguments: `EArgList(head, tail)` terminated by `EArgNil`
- record fields: `EField(fname, val_e, rest)` terminated by `EFieldNil`
- match arms: `EArm(pat, body, rest)` terminated by `EArmNil`

Bootstrap-sensitive helpers rely on these shapes:

- `count_args(...)`
- `compile_args(...)`
- `compile_record_fields(...)`
- `add_field_name_consts_loop(...)`
- `reverse_match_arms_loop(...)`
- `compile_match_arms(...)`

## What is contract-sensitive

The following are currently contract-sensitive for bootstrap:

- nested variant pattern compilation through `PVariantP`
- record field-name collection through `EField`
- match-arm traversal through `EArm`
- record-field access in multi-argument calls

The expanded v6.2.0 regression and bootstrap tests were added specifically to keep
these shapes stable.

## What is not contract-sensitive

The following are implementation details, not contract commitments:

- helper function names such as `reverse_match_arms_loop`
- exact recursion order, as long as emitted bytecode remains bootstrap-stable
- whether list reversal happens during parse-time or codegen-time

## Maintenance rule

When editing `compiler.fav`, prefer this rule:

- if a variant has multiple logical fields and the code path is bootstrap-sensitive, use payload-record access explicitly

This is the safer default for:

- codegen
- serializer-adjacent helpers
- AST traversals that feed codegen or constant emission
