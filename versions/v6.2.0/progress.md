# v6.2.0 Progress

## Phase A
- [x] A-1: Define the `FvcArtifact` serialization format
- [x] A-2: Inspect the output shape of `compiler.fav` and `hello.fav`
- [x] A-3: Confirm self-compilation feasibility for `compiler.fav`
- [x] A-4: Lock the implementation approach for Phase B

## Phase B
- [x] B-1: Introduce `FnEntry` in `compiler.fav`
- [x] B-2: Introduce `Artifact`
- [x] B-3: Change `compile()` to return `Artifact`
- [x] B-4: Implement `serialize_artifact(a: Artifact) -> List<Int>`
- [x] B-5: Emit artifact bytes from `main()` via stdout
- [x] B-6: Verify `fav check fav/self/compiler.fav`
- [x] B-7: Verify `fav run compiler.fav -- hello.fav`

## Phase C
- [x] C-1: Implement `FvcArtifact::from_bytes(bytes: &[u8])`
- [x] C-2: Verify compatibility between self-host serialization and Rust loading

## Phase D
- [x] D-1: Isolate the Stage 2 design/codegen gaps
- [x] D-2: Repair broken self-host codegen and fold fixes into `compiler.fav`
- [x] D-3: Make `fav run compiler.fav -- compiler.fav compiler.fvc` succeed
- [x] D-4: Make `compiler.fvc` loadable by the Rust VM

## Phase E
- [x] E-1: Wire Stage 3 into Rust tests
- [x] E-2: Verify `bytecode_A == bytecode_B`
- [x] E-3: Pass `cargo test bootstrap_full_self_hosting`

## Phase F
- [x] F-1: Pass full `cargo test`
- [x] F-2: Update `versions/v6.2.0/tasks.md`
- [x] F-3: Update `memory/MEMORY.md`
- [x] F-4: Commit as `feat: full bootstrap verified - Favnir compiler bootstraps itself (v6.2.0)`

## Phase G
- [x] G-1: Reduce the `collect { helper(...) }` bootstrap exception scope
- [x] G-2: Add regressions for nested variant guards and related semantic edges
- [x] G-3: Broaden bootstrap comparison with `match` / `collect` / record-heavy input
- [x] G-4: Document the self-host internal AST contract used by bootstrap-sensitive code

## Phase H
- [x] H-1: Add negative regressions for the narrowed `collect { helper(...) }` exception
- [x] H-2: Add bootstrap comparison for closure capture + `for`-inside-`collect`
- [x] H-3: Expand self-host artifact regressions for capture selection, nested match fallthrough, guarded match arms, and nested call lowering

## Phase I
- [x] I-1: Classify the remaining `fav/tmp` diffs as throwaway test outputs for v6.2.0
- [x] I-2: Re-run the focused self-host validation slice serially and note the memory issue with parallel ignored runs
