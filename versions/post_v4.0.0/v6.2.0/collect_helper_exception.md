# Favnir v6.2.0 Collect-Helper Exception Note

Date: 2026-05-25

## Summary

The current `collect { helper(...) }` exception was reduced, but not fully removed, in v6.2.0.

Current state:

- Rust checker permits helper-based `yield` only for ordinary function definitions that are directly invoked as the tail expression of a `collect` block.
- The exception is no longer widened across `trf`, `test`, or `bench` bodies.

## Why it still exists

`fav/self/compiler.fav` currently uses:

- `scan_collect(chars: List<String>) -> Bool`
- `scan_entries(chars) = collect { scan_collect(chars) }`

This shape lets the self-host lexer accumulate tokens through `collect` / `yield` without rebuilding lists through repeated structural concatenation.

Earlier attempts to remove the helper pattern entirely ran into one or both of these problems:

1. type-check friction:
   `yield` is syntactically inside a helper function rather than directly inside the `collect` block
2. bootstrap cost:
   rewriting the lexer into explicit list construction increases allocation and/or recursion pressure enough to make the self-host bootstrap less reliable

## v6.2.0 decision

For v6.2.0 the safer tradeoff is:

- keep the helper-based lexer shape in `compiler.fav`
- keep the exception explicit and narrow
- regression-test the behavior
- document that this is a bootstrap-support rule, not a hidden semantic default

## What was reduced

Before narrowing, helper discovery scanned more item kinds.

After narrowing, helper discovery applies only to:

- ordinary function definitions (`fn`)

It does not intentionally authorize helper-yield behavior through:

- `trf`
- `test`
- `bench`

## Why full removal is deferred

Full removal would require one of:

1. rewriting the self-host lexer away from helper-based `yield`
2. introducing a more principled context-sensitive collect/yield rule
3. adding a cheaper self-host list-building strategy that does not depend on helper-yield accumulation

All three are larger than the intended v6.2.0 hardening scope.

## Working conclusion

The exception remains a known bootstrap compatibility rule.

It is now:

- explicit
- narrower than before
- regression-covered
- documented as a temporary authority gap rather than treated as normal language ownership
