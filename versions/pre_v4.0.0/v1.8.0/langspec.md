# Favnir v1.8.0 Language Specification

## Task<T> Parallel API

### New builtins

| Builtin | Signature | Behaviour |
|---|---|---|
| `Task.all(tasks)` | `List<Task<T>> -> Task<List<T>>` | Runs all tasks, returns all results as a List |
| `Task.race(tasks)` | `List<Task<T>> -> Task<T>` | Returns the first task's result |
| `Task.timeout(task, ms)` | `Task<T> -> Int -> Task<Option<T>>` | Returns `some(value)` in v1.8.0 (synchronous); full timeout in future |

All three are synchronous and transparent at runtime in v1.8.0 (no real concurrency).

### Example

```favnir
bind tasks <- collect {
    yield Task.run(|| compute(1));
    yield Task.run(|| compute(2));
}
bind results <- Task.all(tasks)
IO.println_int(List.length(results))
```

---

## async fn main()

`async fn main() -> Unit !Io` is now a valid program entry point.

```favnir
public async fn main() -> Unit !Io {
    bind msg <- greet("world")
    IO.println(msg)
}
```

The checker registers `main` as `Fn([], Task<Unit>)`. At runtime, `Task<Unit>` is
transparent so the VM driver runs it identically to a plain `main() -> Unit`.

---

## chain + Task<T>

`chain x <-` now unwraps both `Task<X>` and the inner `Result<T,E>` or `Option<T>`.

```favnir
public fn fetch() -> Task<Option<Int>> {
    Task.run(|| Option.some(42))
}

public fn main() -> Option<Int> {
    chain x <- fetch()   // unwraps Task<Option<Int>> → Int
    Option.some(x)
}
```

The chain context type is checked against the inner type after stripping `Task<_>`.

---

## Coverage: Function-level Report

`fav test --coverage` now emits a per-function breakdown after the line-level summary:

```
coverage: src/main.fav
  lines covered: 12 / 15 (80.0%)
  uncovered:     lines 8, 11, 14

function coverage:
  fn main                          3/3  (100%) [full]
  fn helper                        2/3  (67%)  [partial]
  fn unused_fn                     0/2  (0%)   [none]
```

### --coverage-report \<dir\>

```
fav test --coverage --coverage-report ./coverage_out
```

Writes `<dir>/coverage.txt` containing the full line + function report for all
source files. The directory is created if it does not exist.

---

## fav bench

The `bench` keyword defines a benchmark body. Benchmark files conventionally use
the `.bench.fav` extension.

### Syntax

```favnir
bench "description" {
    body_expression
}
```

The body is any block expression. It may reference top-level functions.
`bench` items cannot have visibility modifiers.

### CLI

```
fav bench [file]
fav bench [file] --filter <substring>
fav bench [file] --iters <N>
```

| Flag | Default | Description |
|---|---|---|
| `--filter` | (none) | Only run benchmarks whose description contains the substring |
| `--iters` | 100 | Number of timed iterations per benchmark |

### Output

```
running 3 benchmarks (100 iterations each)

bench  fib(10)                                    0.42 µs/iter  (100  math.bench.fav)
bench  fib(15)                                    4.71 µs/iter  (100  math.bench.fav)
bench  factorial(10)                              0.19 µs/iter  (100  math.bench.fav)

bench result: ok. 0 filtered
```

### Compilation

Each `bench "desc" { body }` compiles to a function named `$bench:desc` in the
artifact. The bench runner calls it N+1 times (1 warmup + N timed).

---

## Error codes

| Code | Description |
|---|---|
| E061 | `Task.all` / `Task.race` called with an empty list |
