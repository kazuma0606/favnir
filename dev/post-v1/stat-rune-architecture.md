# Stat Rune Architecture

## Positioning

`stat` is a better parent rune than `sample` for Favnir.

Reason:

- `sample` sounds like a narrow synthetic-data helper
- Favnir needs a broader space that includes:
  - probability distributions
  - synthetic data generation
  - row and CSV sampling
  - simulation-oriented data generation

So the recommended family is:

- `stat`
- optionally later:
  - `stat.dist`
  - `stat.sample`
  - `stat.sim`

For the first implementation, a single `stat` rune is enough.

## Scope

`stat` should cover three related areas.

### 1. Distribution-driven generation

Examples:

- normal distribution
- uniform distribution
- weighted choice
- t-distribution
- chi-square distribution
- Bernoulli / boolean-like generation

This is useful for test inputs, notebook experiments, and simulation-heavy pipelines.

### 2. Sampling from existing data

Examples:

- sample rows from a list
- sample rows from CSV-derived data
- stratified sampling
- deterministic seeded sampling
- reservoir sampling later

This is useful when real data exists but only partial or representative subsets are needed.

### 3. Simulation-oriented synthetic data

Examples:

- generate domain-shaped synthetic users/orders/events
- generate heavy-tailed or biased input
- generate rare-event scenarios
- stress-test pipelines with controlled distributions

This is useful for pipeline design before production connectors exist.

## Why `stat` Fits Favnir

Favnir is not just a language for pure computation.
It is increasingly a language for:

- data transformation
- validation
- notebook-based exploration
- explainable pipelines

`stat` fits this direction better than a narrow `sample` rune because it naturally supports both:

- synthetic generation
- real-data sampling

This also makes it useful for Veltra notebook workflows.

## Core API Shape

A good first-pass API is:

### Primitive generation

```fav
Stat.int(min: Int, max: Int, seed: Int?) -> Int
Stat.float(min: Float, max: Float, seed: Int?) -> Float
Stat.bool(seed: Int?) -> Bool
Stat.choice<T>(values: List<T>, seed: Int?) -> T
Stat.weighted<T>(values: List<(T, Float)>, seed: Int?) -> T
Stat.string(len: Int, seed: Int?) -> String
```

These are low-level building blocks.

### Distributions

```fav
Stat.normal(mean: Float, stddev: Float, seed: Int?) -> Float
Stat.uniform(min: Float, max: Float, seed: Int?) -> Float
Stat.t(df: Int, seed: Int?) -> Float
Stat.chi_square(df: Int, seed: Int?) -> Float
```

This is where Favnir becomes more interesting than a simple faker library.

### Structured generation

```fav
Stat.one<T>(seed: Int?) -> T
Stat.list<T>(count: Int, seed: Int?) -> List<T>
Stat.rows<T>(count: Int, seed: Int?) -> List<T>
```

This is the type-driven synthetic data layer.

Example:

```fav
type User = {
    id: Int
    name: String
    email: String
}

bind user <- Stat.one<User>(seed: 42)
bind users <- Stat.list<User>(100, seed: 42)
```

## Sampling API

Sampling should be a first-class part of `stat`.

Suggested first API:

```fav
Stat.sample_rows<T>(rows: List<T>, n: Int, seed: Int?) -> List<T>
Stat.sample_one<T>(rows: List<T>, seed: Int?) -> T?
Stat.sample_csv_rows(rows: List<Map<String>>, n: Int, seed: Int?) -> List<Map<String>>
Stat.stratified<T, K>(rows: List<T>, key: T -> K, n: Int, seed: Int?) -> List<T>
```

This allows:

- random row subsets
- preview datasets
- representative notebook demos
- light-weight ETL testing

## Simulation API

The most interesting higher-level direction is simulation.

Suggested shape:

```fav
Stat.simulate<T>(count: Int, model: SimModel<T>, seed: Int?) -> List<T>
Stat.monte_carlo<T>(count: Int, model: SimModel<T>, seed: Int?) -> List<T>
```

Important note:

- Monte Carlo is not mainly about producing a single “more accurate sample”
- it is about producing data whose statistical behavior follows a model over many trials

So Monte Carlo should be described as:

- model-driven synthetic generation
- distribution-driven simulation
- scenario testing

rather than just “better randomness.”

## Recommended Semantics for Seeds

Seeds matter a lot.

Requirements:

- seeded generation should be reproducible
- tests should be deterministic by default when seed is provided
- notebook examples should be shareable and replayable

So every public generator/sampler should either:

- take `seed: Int?`
- or derive from a configurable `Stat.seed(...)` context later

## Effect Model

There are two reasonable ways to position `stat`.

### Option A: Pure-by-seed

If the seed is explicit, generation can be treated as pure.

Example interpretation:

- `Stat.normal(..., seed: 42)` always returns the same value

This is attractive for tests and explainability.

### Option B: Random effect

If no explicit seed exists, generation could be effectful.

Example:

- `!Random`

Recommendation:

- first implementation should prefer pure-by-seed semantics
- random-effect modelling can come later if needed

This keeps the API simpler and more reproducible.

## Relationship to `validate`

`stat` and `validate` work especially well together.

Typical pipeline:

```fav
bind rows <- Stat.list<UserRow>(1000, seed: 42)
bind result <- rows |> ValidateRows
```

This enables:

- testing pipelines without production data
- stress-testing validators
- notebook-first exploration
- CI-ready synthetic datasets

`validate` and `stat` together form a strong pair of early Pure Favnir runes.

## Veltra Value

Veltra benefits directly from `stat`.

Notebook users want to:

- prototype without connectors
- simulate realistic row shapes
- preview distributions
- run validation and transforms early
- sample large datasets into smaller dev-sized subsets

`stat` supports all of this.

So `stat` is not just a stdlib utility.
It is also part of the product onboarding and experimentation story.

## Recommended Implementation Order

1. primitive seeded generators
   - `int`, `float`, `bool`, `choice`, `string`
2. simple distributions
   - `uniform`, `normal`
3. row/list sampling
   - `sample_rows`, `sample_one`
4. type-driven structured generation
   - `one<T>`, `list<T>`
5. richer distributions
   - `t`, `chi_square`
6. simulation layer
   - `simulate`, `monte_carlo`
7. stratified/reservoir sampling

## Summary

Recommended direction:

- use `stat` as the parent rune
- treat synthetic generation and real-data sampling as one family
- keep seeds explicit and reproducible
- support distributions and later Monte Carlo/simulation
- pair `stat` closely with `validate`, tests, and Veltra notebooks

This gives Favnir a more distinctive data-language identity than a narrow `sample` rune would.
