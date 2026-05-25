# Favnir Self-Host Maturity Note

Date: 2026-05-25

## Current assessment

Favnir is currently at `Level 3 / 5`:
`Bootstrap Verified`.

What this means today:

- `compiler.fav` can compile itself.
- The produced compiler artifact can recompile the same workload.
- The bootstrap check reaches byte-for-byte agreement for the verified path.
- Full `cargo test` passes alongside the bootstrap test.

This is a strong self-host milestone, but it is not yet "full self-host" in the strict sense.
Rust still remains the operational authority for parts of the runtime and language-processing stack.

## The 5 levels

### Level 1: Hosted Prototype

- Rust is the clear source of truth.
- The self-host compiler is partial or experimental.
- Small programs may work, but self-compilation is not yet meaningful.

### Level 2: Self-Compile Capable

- The self-host compiler can compile itself.
- Successful self-compilation exists, but reproducibility and stability are not yet strong guarantees.

### Level 3: Bootstrap Verified

- `source compiler -> compiler artifact -> same workload` is verified.
- Recompiled output matches the original checked output.
- Self-reproduction is proven for the tested bootstrap path.

Favnir is here now.

### Level 4: Self-Hosted Authority

- Day-to-day compiler evolution happens primarily in the self-host compiler.
- Rust becomes a narrower bootstrap kernel: loader, VM, platform glue, minimal host support.
- Parser/checker/lowering/runtime semantics are no longer primarily "owned" by Rust.

### Level 5: Minimal-Rust / Full Self-Host

- The language, compiler, and primary tooling are effectively self-owned.
- Rust is retained only where it is intentionally the safer or more practical substrate.
- Self-host is the default authority, not just a verified secondary implementation.

## Important practical nuance

"Full self-host" is not the only sensible end state.

For an individual developer or a small team, some domains are realistically better left in Rust even if the compiler itself becomes strongly self-hosted.
This is especially true where mistakes have a high safety cost or where the implementation burden is specialist-heavy.

Examples:

- security-sensitive code
- cryptography
- low-level binary parsing and serialization boundaries
- network protocol implementations with high robustness requirements
- memory-sensitive runtime internals

In those areas, keeping a Rust implementation is not a failure of self-hosting.
It is often the safer engineering decision.

## Recommended interpretation for Favnir

Favnir should aim for:

- self-hosted authority for the compiler pipeline
- selective Rust dependence for high-risk infrastructure

That target is compatible with calling the project "strongly self-hosted" later, even if it never becomes "Rust-free".

In practice, the desirable split is:

- self-host owns syntax, AST lowering, semantic checks, code generation, and bootstrap evolution
- Rust owns the narrowest possible trusted substrate where safety and implementation maturity matter most

## What blocks Level 4 today

- Rust still defines important checker/runtime behavior that the self-host side follows rather than owns.
- The artifact loader and VM contract still live primarily in Rust.
- Some bootstrap-support semantics are accepted because Rust recognizes them.

## Direction from here

To move from Level 3 toward Level 4:

1. Reduce parser/checker/lowering semantic gaps between Rust and `compiler.fav`.
2. Make self-host changes the default path for compiler evolution.
3. Document the artifact/IR/opcode contract explicitly.
4. Keep Rust as a small trusted kernel, not the broad language authority.

## Working conclusion

Favnir is already beyond a prototype self-host compiler.
It is now bootstrap-verified.

The next goal should not be "remove Rust everywhere".
The better goal is:

`self-host where language ownership matters, Rust where safety and robustness matter most`.
