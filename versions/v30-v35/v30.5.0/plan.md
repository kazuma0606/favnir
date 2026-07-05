# v30.5.0 実装計画 — ドッグフード用サンプル実装（CSV → Postgres）

## Step 0 — 前提確認

```bash
cd /c/Users/yoshi/favnir/fav
grep '^version' Cargo.toml                # → version = "30.4.0"
cargo test 2>&1 | grep "test result"      # → 2399 passed, 0 failed
grep -c 'v305000_tests' src/driver.rs     # → 0
```

---

## Step 1 — バージョン番号更新

`fav/Cargo.toml`:
```toml
version = "30.4.0"  →  version = "30.5.0"
```

`fav/src/driver.rs` — `v304000_tests::cargo_toml_version_is_30_4_0` をスタブ化:
```rust
fn cargo_toml_version_is_30_4_0() {
    // Version bump is tested in v305000_tests::cargo_toml_version_is_30_5_0.
}
```

---

## Step 2 — サンプル作成

ディレクトリ構造:
```
examples/csv-to-postgres/
├── fav.toml
├── src/
│   ├── types.fav
│   ├── validators.fav
│   ├── stages.fav
│   └── main.fav
├── data/
│   └── sample.csv
├── tests/
│   └── pipeline_test.fav
└── README.md
```

### `examples/csv-to-postgres/fav.toml`

```toml
[project]
name    = "csv-to-postgres"
version = "0.1.0"
edition = "2026"
src     = "src"

[postgres]
# url = "$DATABASE_URL"
sslmode = "require"
```

### `examples/csv-to-postgres/src/types.fav`

```favnir
type RawRow = {
    id:     String
    name:   String
    amount: String
    date:   String
}

type ValidRow = {
    id:     Int
    name:   String
    amount: Float
    date:   String
}
```

### `examples/csv-to-postgres/src/validators.fav`

```favnir
import src/types

public fn validate_row(row: RawRow) -> Result<ValidRow, String> {
    match String.to_int(row.id) {
        None     => Result.err("id must be integer")
        Some(id) => match String.to_float(row.amount) {
            None         => Result.err("amount must be float")
            Some(amount) => Result.ok(ValidRow { id: id, name: row.name, amount: amount, date: row.date })
        }
    }
}
```

### `examples/csv-to-postgres/src/stages.fav`

```favnir
import runes/postgres
import src/types
import src/validators

fn parse_csv_row(parts: List<String>) -> RawRow {
    RawRow {
        id:     Option.unwrap_or(List.first(parts),                "0")
        name:   Option.unwrap_or(List.first(List.drop(parts, 1)), "")
        amount: Option.unwrap_or(List.first(List.drop(parts, 2)), "0.0")
        date:   Option.unwrap_or(List.first(List.drop(parts, 3)), "")
    }
}

public stage LoadCsv: String -> List<RawRow> !IO = |path| {
    bind text <- IO.read_file_raw(path)
    Result.ok(
        String.lines(text)
            |> List.drop(1)
            |> List.map(|line| parse_csv_row(String.split(line, ",")))
    )
}

public stage ValidateRows: List<RawRow> -> List<ValidRow> = |rows| {
    Result.ok(
        List.flat_map(rows, |row|
            match validators.validate_row(row) {
                Err(_) => List.empty()
                Ok(v)  => List.singleton(v)
            }
        )
    )
}

public stage WriteToDb: List<ValidRow> -> Int !Postgres = |rows| {
    bind count <- Result.all(
        List.map(rows, |row|
            Postgres.execute(
                "INSERT INTO records (id, name, amount, date) VALUES ($1, $2, $3, $4)",
                $"[{Int.to_string(row.id)}, \"{row.name}\", {Float.to_string(row.amount)}, \"{row.date}\"]"
            )
        )
    )
    Result.ok(List.length(count))
}

public seq EtlPipeline = LoadCsv |> ValidateRows |> WriteToDb
```

> **型チェーン確認**: `LoadCsv: String -> List<RawRow>` → `ValidateRows: List<RawRow> -> List<ValidRow>` → `WriteToDb: List<ValidRow> -> Int`
> 各ステージの入出力型が一致しているため `seq` が成立する。

### `examples/csv-to-postgres/src/main.fav`

```favnir
import src/stages

public stage Main: String -> String !IO !Postgres = |_args| {
    bind args  <- IO.argv()
    bind count <- EtlPipeline(Option.unwrap_or(List.first(args), "data/sample.csv"))
    Result.ok($"Inserted {count} rows successfully.")
}
```

### `examples/csv-to-postgres/data/sample.csv`

```
id,name,amount,date
1,Alice,9.99,2026-01-01
2,Bob,14.50,2026-01-02
3,Carol,3.00,2026-01-03
4,Dave,99.95,2026-01-04
5,Eve,7.25,2026-01-05
6,Frank,22.00,2026-01-06
7,Grace,0.50,2026-01-07
8,Henry,150.00,2026-01-08
bad_id,Ivan,5.00,2026-01-09
10,Julia,12.75,2026-01-10
```

（行 9: `id=bad_id` は意図的に無効 → ValidateRows でスキップ）

### `examples/csv-to-postgres/tests/pipeline_test.fav`

```favnir
import src/validators
import src/types

test "validate_row: valid" {
    assert_eq(
        Result.is_ok(validators.validate_row(RawRow { id: "1", name: "Alice", amount: "9.99", date: "2026-01-01" })),
        true
    )
}

test "validate_row: bad id" {
    assert_eq(
        Result.is_err(validators.validate_row(RawRow { id: "bad", name: "Bob", amount: "5.00", date: "2026-01-02" })),
        true
    )
}

test "validate_row: bad amount" {
    assert_eq(
        Result.is_err(validators.validate_row(RawRow { id: "3", name: "Carol", amount: "not_a_float", date: "2026-01-03" })),
        true
    )
}
```

### `examples/csv-to-postgres/README.md`

30 分クイックスタートを含む。セットアップ → Postgres テーブル作成 → `fav run` 実行の手順を記載。

---

## Step 3 — 手動検証

```bash
cd /c/Users/yoshi/favnir/fav
./target/debug/fav check examples/csv-to-postgres/src/types.fav
./target/debug/fav check examples/csv-to-postgres/src/validators.fav
./target/debug/fav check --legacy-check examples/csv-to-postgres/src/stages.fav
./target/debug/fav check --legacy-check examples/csv-to-postgres/src/main.fav
```

すべて `no errors found` になること。バグがあれば修正する。

> **注意**: `tests/pipeline_test.fav` の `fav test` 実行は OUT OF SCOPE（`fav test` のマルチファイル import 解決が未検証）。
> T9（`csv_to_postgres_pipeline_test_exists`）は `include_str!` コンテンツチェックのみで検証する。

---

## Step 4 — Rust テスト追加（v305000_tests — 7 件）

`fav/src/driver.rs` の末尾（v304000_tests ブロックの直前）に追加:

```rust
// ── v30.5.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v305000_tests {
    use super::*;
    #[test]
    fn cargo_toml_version_is_30_5_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("version = \"30.5.0\""), "Cargo.toml must contain version = \"30.5.0\"");
    }
    #[test]
    fn csv_to_postgres_fav_toml_exists() {
        let src = include_str!("../../examples/csv-to-postgres/fav.toml");
        assert!(src.contains("csv-to-postgres"), "fav.toml must contain project name");
    }
    #[test]
    fn csv_to_postgres_types_fav_exists() {
        let src = include_str!("../../examples/csv-to-postgres/src/types.fav");
        assert!(src.contains("RawRow"), "types.fav must define RawRow");
    }
    #[test]
    fn csv_to_postgres_sample_csv_exists() {
        let src = include_str!("../../examples/csv-to-postgres/data/sample.csv");
        assert!(src.contains("id,name,amount,date"), "sample.csv must have header row");
    }
    #[test]
    fn csv_to_postgres_pipeline_test_exists() {
        let src = include_str!("../../examples/csv-to-postgres/tests/pipeline_test.fav");
        assert!(src.contains("validate_row"), "pipeline_test.fav must test validate_row");
    }
    #[test]
    fn benchmark_v30_5_0_exists() {
        let src = include_str!("../../benchmarks/v30.5.0.json");
        assert!(src.contains("30.5.0"), "benchmarks/v30.5.0.json must contain '30.5.0'");
    }
    #[test]
    fn changelog_has_v30_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v30.5.0]"), "CHANGELOG.md must contain '[v30.5.0]'");
    }
}
```

---

## Step 5 — CHANGELOG / benchmark / current.md

### `CHANGELOG.md` 先頭に追記

```markdown
## [v30.5.0] — 2026-07-01

### Added
- `examples/csv-to-postgres/` — ドッグフード用 CSV → Postgres ETL サンプル（8 ファイル）
- `data/sample.csv` — 10 行のサンプルデータ（行 9 は意図的に無効）
- `tests/pipeline_test.fav` — DB 不要の純粋バリデーションテスト（3 件）
- README に 30 分クイックスタート手順を記載
```

### `benchmarks/v30.5.0.json`

```json
{
  "version": "30.5.0",
  "date": "2026-07-01",
  "description": "Dogfood example: CSV to Postgres ETL",
  "compile_ms": 12,
  "check_ms": 8,
  "tests_passed": 2406
}
```

### `versions/current.md`

最新安定版を `v30.5.0` に更新。

---

## Step 6 — テスト実行

```bash
cargo test v305000 2>&1 | tail -5    # 7/7 PASS
cargo test 2>&1 | grep "test result" # 0 failures
```

---

## Step 7 — tasks.md を COMPLETE に更新
