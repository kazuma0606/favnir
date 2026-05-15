# Favnir v3.5.0 Language Specification

## New in v3.5.0: `gen` rune — Type-Driven Data Generation

---

## 1. `Random.seed(n: Int) -> Unit`

Seeds the global RNG for deterministic execution. All subsequent `Random.*`
and `Gen.*` calls will produce the same values for the same seed.

```favnir
Random.seed(42);
bind x <- Random.int(1, 100)  // always the same value for seed 42
```

---

## 2. `Gen` VM Primitives

### `Gen.string_val(len: Int) -> String !Random`

Generates a random alphanumeric string of the given length.

```favnir
bind s <- Gen.string_val(8)  // e.g. "aB3kRq7Z"
```

### `Gen.one_raw(type_name: String) -> Map<String, String> !Random`

Generates one random row for the named type. Each field is generated
according to its type:

| Favnir type | Generated value |
|-------------|----------------|
| `Int`       | Random integer in `[-1000, 1000]` |
| `Float`     | Random float in `[0.0, 1.0)` |
| `Bool`      | `"true"` or `"false"` |
| `String`    | 8-character alphanumeric string |
| `Option<T>` | 50% empty string (None), 50% value for T |

```favnir
type User = { id: Int name: String }

bind row <- Gen.one_raw("User")
// row = { "id": "42", "name": "aB3kRq7Z" }
```

### `Gen.list_raw(type_name: String, n: Int) -> List<Map<String, String>> !Random`

Generates `n` random rows. Pair with `Random.seed` for reproducibility.

```favnir
Random.seed(42);
bind users <- Gen.list_raw("User", 100)
```

### `Gen.simulate_raw(type_name: String, n: Int, noise: Float) -> List<Map<String, String>> !Random`

Generates `n` rows with intentional corruption. `noise` is a fraction in
`[0.0, 1.0]` — e.g. `0.1` corrupts ~10% of field values.

Corruption patterns:
- `Int`/`Float` fields → `"NaN"`
- `Bool` fields → `"maybe"`
- `String` fields → `""`

```favnir
Random.seed(1);
bind dirty <- Gen.simulate_raw("User", 1000, 0.05)  // 5% noise
```

### `Gen.profile_raw(type_name: String, data: List<Map<String, String>>) -> GenProfile`

Measures data quality against the type schema. Returns a `GenProfile` record.

```favnir
bind prof <- Gen.profile_raw("User", dirty)
IO.println($"Valid rate: {prof.rate}")
```

---

## 3. `GenProfile` Type

Pre-registered built-in type (no declaration needed):

```favnir
type GenProfile = {
    total:   Int    // total rows
    valid:   Int    // rows where all fields pass type validation
    invalid: Int    // rows with at least one invalid field
    rate:    Float  // valid / total (0.0 if total == 0)
}
```

---

## 4. `runes/gen/gen.fav` Public API

The `gen` rune wraps the VM primitives with a clean Favnir API:

| Function | Signature | Description |
|----------|-----------|-------------|
| `gen.int_val` | `(Int, Int) -> Int !Random` | Random integer in [min, max] |
| `gen.float_val` | `() -> Float !Random` | Random float in [0.0, 1.0) |
| `gen.bool_val` | `() -> Bool !Random` | Random boolean |
| `gen.string_val` | `Int -> String !Random` | Random alphanumeric of given length |
| `gen.choice` | `List<String> -> Option<String> !Random` | Pick a random element |
| `gen.one` | `String -> Map<String,String> !Random` | Generate one row |
| `gen.list` | `(String, Int, Int) -> List<...> !Random` | Generate N rows with seed |
| `gen.simulate` | `(String, Int, Float, Int) -> List<...> !Random` | N rows with noise + seed |
| `gen.profile` | `(String, List<...>) -> GenProfile` | Profile data quality |

```favnir
import rune "gen"

type User = { id: Int name: String age: Int }

public fn main() -> Unit !Io !Random {
    bind users <- gen.list("User", 100, 42)
    bind report <- gen.profile("User", users)
    IO.println($"Valid: {report.valid}/{report.total}")
}
```

---

## 5. `fav check --sample N`

```bash
fav check pipeline.fav --sample 100
```

Generates `N` synthetic rows for the first record type in the file and
verifies the pipeline runs without runtime errors.

```
Generating 100 synthetic rows for type 'User'...
Running pipeline with synthetic data...
  ok: all 100 rows processed without errors
  (use --sample to test with real data for integration verification)
```

---

## 6. Typical Workflow

```bash
# Step 1: Infer type from real data
fav infer customers.csv --out schema/customer.fav

# Step 2: Verify pipeline with synthetic data before obtaining real data
fav check pipeline.fav --sample 500

# Step 3: Generate noisy data to test cleansing logic
fav run cleanse_demo.fav
```
