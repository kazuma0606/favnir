# v30.5.0 仕様書 — ドッグフード用サンプル実装（CSV → Postgres）

## 概要

`examples/csv-to-postgres/` に完全な ETL パイプラインのサンプルを追加する。
実際のユーザーが「30 分で動かせる」レベルの完成度を目指す。

---

## 背景

v30.1〜v30.4 でビルド軽量化・テンプレート改善・フィクスチャ検証を行った。
これらの成果を活かし、実際に動作するサンプルコードを `examples/` に置く。

ロードマップ定義のパイプライン処理:
1. `LoadCsv` — CSV を読み込み `RawRow` のリストに変換
2. `ValidateRows` — 型変換・バリデーション
3. `WriteToDb` — バリデーション済みデータを Postgres に書き込み

---

## スコープ

### IN SCOPE

- `examples/csv-to-postgres/` ディレクトリ（8 ファイル）
  - `fav.toml`
  - `src/types.fav`
  - `src/validators.fav`
  - `src/stages.fav`
  - `src/main.fav`
  - `data/sample.csv`（10 行のサンプルデータ）
  - `tests/pipeline_test.fav`（DB 不要の純粋テスト）
  - `README.md`（30 分クイックスタート）
- `v305000_tests`（7 件）Rust テスト追加
- CHANGELOG / benchmark / current.md 更新

### OUT OF SCOPE

- 実際の DB 接続テスト（`fav test` は純粋バリデーションテストのみ）
- site/ MDX 更新
- 1000 行 CSV（git 肥大化防止のため 10 行）

---

## E0023 制約（v30.4.0 調査結果）

`fav check`（non-legacy）は `IO.*` / `Postgres.*` の ambient 呼び出しを E0023 として拒否する。
stages.fav / main.fav は IO・Postgres ステージを含むため **`fav check --legacy-check`** を使用する。

| ファイル | `fav check` モード |
|---|---|
| types.fav | `fav check`（non-legacy）✓ |
| validators.fav | `fav check`（non-legacy）✓ |
| stages.fav | `fav check --legacy-check`（IO/Postgres stages）|
| main.fav | `fav check --legacy-check`（IO/Postgres 呼び出し）|
| tests/pipeline_test.fav | `fav test`（純粋テスト）✓ |

---

## ファイル設計

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

### `src/types.fav`

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

### `src/validators.fav`（純粋関数 — no Rune import）

エラー型は `String`（bind 連鎖の error 型統一）:

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

### `src/stages.fav`（IO + Postgres — `fav check --legacy-check`）

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

> **注意**: `ValidateRows` は `List.flat_map` + `List.empty()`/`List.singleton()` でエラー行をスキップし `List<ValidRow>` を返す。
> `List.filter_map` は VM に未実装のため使用しない。
> `seq` 内の型チェーン: `String -> List<RawRow> -> List<ValidRow> -> Int` で一致。

### `src/main.fav`

```favnir
import src/stages

public stage Main: String -> String !IO !Postgres = |_args| {
    bind args  <- IO.argv()
    bind count <- EtlPipeline(Option.unwrap_or(List.first(args), "data/sample.csv"))
    Result.ok($"Inserted {count} rows successfully.")
}
```

### `data/sample.csv`（10 行）

```csv
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

（行 9 は `id=bad_id` で意図的に無効 → ValidateRows でスキップされる）

### `tests/pipeline_test.fav`（DB 不要）

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

---

## テスト設計（v305000_tests — 7 件）

| # | テスト名 | 確認内容 |
|---|---------|---------|
| 1 | `cargo_toml_version_is_30_5_0` | `Cargo.toml` に `version = "30.5.0"` |
| 2 | `csv_to_postgres_fav_toml_exists` | `examples/csv-to-postgres/fav.toml` に `csv-to-postgres` |
| 3 | `csv_to_postgres_types_fav_exists` | `src/types.fav` に `RawRow` |
| 4 | `csv_to_postgres_sample_csv_exists` | `data/sample.csv` に `id,name,amount,date` |
| 5 | `csv_to_postgres_pipeline_test_exists` | `tests/pipeline_test.fav` に `validate_row` |
| 6 | `benchmark_v30_5_0_exists` | `benchmarks/v30.5.0.json` に `30.5.0` |
| 7 | `changelog_has_v30_5_0` | `CHANGELOG.md` に `[v30.5.0]` |

---

## 完了条件

- `Cargo.toml` version = "30.5.0"
- `examples/csv-to-postgres/` — 8 ファイル（fav.toml + 4 .fav + sample.csv + README.md + pipeline_test.fav）
- `fav check examples/csv-to-postgres/src/types.fav` → no errors found
- `fav check examples/csv-to-postgres/src/validators.fav` → no errors found
- `fav check examples/csv-to-postgres/src/stages.fav` → no errors found（v30.5.0 で TrfDef の E0023/E0025 免除）
- `fav check examples/csv-to-postgres/src/main.fav` → no errors found
- `cargo test v305000` — 7/7 PASS
- `cargo test` — 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v30.5.0]` セクション
- `benchmarks/v30.5.0.json` 存在
- `versions/current.md` を v30.5.0 に更新
- `tasks.md` が COMPLETE
