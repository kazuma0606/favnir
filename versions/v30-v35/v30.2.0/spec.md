# v30.2.0 Spec — postgres-etl テンプレート v2

**バージョン**: 30.2.0
**日付**: 2026-07-01
**フェーズ**: Real-World Readiness（phase 2/9）
**前バージョン**: v30.1.0（ビルド軽量化）

---

## 概要

`fav new --template postgres-etl` で生成されるプロジェクトを、
現状の 2 ファイル構成（`pipeline.fav` + `main.fav`）から
実用的な 4 ファイル構成に更新する。

生成プロジェクトが `fav check` を通り、型定義・バリデーション・ステージが
それぞれ独立したファイルに整理された状態を目標とする。

---

## 対象コンポーネント

| コンポーネント | 内容 |
|---|---|
| `fav/Cargo.toml` | version `30.1.0` → `30.2.0` |
| `fav/Cargo.lock` | `cargo build` 実行後に変更があればコミットに含める |
| `fav/src/driver.rs` | `create_postgres_etl_project` を 4 ファイル構成に更新 |
| `fav/src/driver.rs` | scaffold 関数 6 件を追加（types / validators / stages / main_v2 / test / readme）|
| `fav/src/driver.rs` | `new_template_postgres_etl_creates_dir` テストを更新（v2 ファイル構成に対応）|
| `fav/src/driver.rs` | `v302000_tests` 6 件追加 |
| `CHANGELOG.md` | `[v30.2.0]` セクション追加 |
| `benchmarks/v30.2.0.json` | ベンチマーク記録（test_count: 2384）|
| `versions/current.md` | 進行中バージョンを `v30.2.0` に更新 |
| `versions/v30-v35/v30.2.0/tasks.md` | 実装完了後 COMPLETE に更新 |

---

## 実装内容

### 生成ファイル構成（v2）

```
my-project/
├── fav.toml                      [project] + [postgres] 設定
├── src/
│   ├── types.fav                 型定義（RawRow / ValidRow / RowError）
│   ├── validators.fav            バリデーションロジック
│   ├── stages.fav                パイプラインステージ
│   └── main.fav                  エントリポイント
├── tests/
│   └── pipeline_test.fav         テストファイル
└── README.md
```

### `src/types.fav`

```favnir
// 入力 CSV の生データ型
type RawRow = {
    id:     String
    name:   String
    amount: String
    date:   String
}

// バリデーション済みの型
type ValidRow = {
    id:     Int
    name:   String
    amount: Float
    date:   String
}

// エラー型（フィールド名とメッセージのみ）
type RowError = {
    field:   String
    message: String
}
```

### `src/validators.fav`

```favnir
import src/types

// String.to_int / String.to_float を使用（VM 確認済み）
public fn validate_row(row: RawRow) -> Result<ValidRow, RowError> {
    match String.to_int(row.id) {
        Err(_) => Result.err({ field: "id", message: "id must be integer" })
        Ok(id) => match String.to_float(row.amount) {
            Err(_) => Result.err({ field: "amount", message: "amount must be float" })
            Ok(amount) => Result.ok({ id: id, name: row.name, amount: amount, date: row.date })
        }
    }
}
```

**注意**: `Int.parse` / `Float.parse` は VM 未実装。VM 確認済みの `String.to_int` / `String.to_float` を使用する。

### `src/stages.fav`

```favnir
import runes/postgres
import src/types
import src/validators

// CSV 1行をパース（List.get は VM 未実装のため Option.unwrap_or + List.drop を使用）
fn parse_csv_row(line: String) -> RawRow {
    {
        id:     Option.unwrap_or(List.first(String.split(line, ",")),                    "0")
        name:   Option.unwrap_or(List.first(List.drop(String.split(line, ","), 1)), "")
        amount: Option.unwrap_or(List.first(List.drop(String.split(line, ","), 2)), "0.0")
        date:   Option.unwrap_or(List.first(List.drop(String.split(line, ","), 3)), "")
    }
}

public stage LoadCsv: String -> List<RawRow> !IO = |path| {
    bind text <- IO.read_file_raw(path)
    Result.ok(
        String.lines(text)
            |> List.drop(1)
            |> List.map(parse_csv_row)
    )
}

// List.map_indexed は VM 未実装のため List.map を使用（idx なし）
public stage ValidateRows: List<RawRow> -> List<ValidRow> = |rows| {
    Result.all(List.map(rows, |row| validators.validate_row(row)))
}

public stage WriteToDb: List<ValidRow> -> Int !Postgres = |rows| {
    bind count <- Result.all(
        List.map(rows, |row|
            Postgres.execute(
                "INSERT INTO records (id, name, amount, date) VALUES ($1, $2, $3, $4)",
                $"[\"{Int.to_string(row.id)}\", \"{row.name}\", \"{Float.to_string(row.amount)}\", \"{row.date}\"]"
            )
        )
    )
    Result.ok(List.length(count))
}

public seq EtlPipeline = LoadCsv |> ValidateRows |> WriteToDb
```

### `src/main.fav`

```favnir
import src/stages

public stage Main: String -> String !IO !Postgres = |_args| {
    bind args  <- IO.argv()
    bind count <- EtlPipeline(Option.unwrap_or(List.first(args), "data/sample.csv"))
    Result.ok($"Inserted {count} rows successfully.")
}
```

**注意**: `List.first` は `Option<String>` を返すため `Option.unwrap_or` でデフォルト値を設定する。

### `tests/pipeline_test.fav`

```favnir
import src/validators
import src/types

test "validate_row: valid input" {
    assert_eq(
        Result.is_ok(validators.validate_row({ id: "1", name: "Alice", amount: "9.99", date: "2026-01-01" })),
        true
    )
}

test "validate_row: invalid id" {
    assert_eq(
        Result.is_err(validators.validate_row({ id: "abc", name: "Bob", amount: "5.00", date: "2026-01-01" })),
        true
    )
}
```

### `README.md`

```markdown
# {name}

Postgres ETL パイプライン — Favnir テンプレート v2

## セットアップ

export DATABASE_URL="postgres://user:pass@localhost:5432/mydb"

## 実行

fav check                              # 型チェック
fav run src/main.fav data/sample.csv  # ETL 実行
fav test                               # テスト実行
```

---

## 既存テストへの影響

### `new_template_postgres_etl_creates_dir`（更新）

現在 `src/pipeline.fav` の存在を検査しているが v2 では生成されなくなる。
新ファイル（types / validators / stages / main / tests / README）の存在確認に変更する。

### `scaffold_postgres_etl_uses_chain`（影響なし）

`scaffold_postgres_etl()` 自体は `cmd_scaffold` 経由で引き続き使用されるため削除しない。
このテストは引き続き通過する（`EtlPipeline` は旧スタイルの `chain` を含む）。

---

## VM プリミティブ確認

本 spec の scaffold コードは以下の VM 実装確認済みプリミティブのみを使用する:

| 関数 | 確認 |
|---|---|
| `String.to_int` | ✓ vm.rs 10772 |
| `String.to_float` | ✓ vm.rs 10785 |
| `Int.to_string` / `Float.to_string` | ✓ vm.rs 10095 |
| `List.first` | ✓ vm.rs 10895 |
| `List.last` | ✓ vm.rs 10908 |
| `List.drop` | ✓ vm.rs（fold の実装内で確認）|
| `List.map` | ✓ vm.rs 3235 |
| `List.length` | ✓ vm.rs 10875 |
| `Option.unwrap_or` | ✓ vm.rs 4154 |
| `Result.all` | ✓（既存テンプレートで使用）|

---

## テスト戦略

### v302000_tests（6 件）

| テスト名 | 検証内容 |
|---|---|
| `cargo_toml_version_is_30_2_0` | `fav/Cargo.toml` が `30.2.0` を含む |
| `postgres_etl_v2_creates_types_fav` | 生成プロジェクトに `src/types.fav` が存在する |
| `postgres_etl_v2_creates_validators_fav` | 生成プロジェクトに `src/validators.fav` が存在する |
| `postgres_etl_v2_creates_stages_fav` | 生成プロジェクトに `src/stages.fav` が存在する |
| `changelog_has_v30_2_0` | `CHANGELOG.md` に `[v30.2.0]` が存在する |
| `benchmark_v30_2_0_exists` | `benchmarks/v30.2.0.json` が `"30.2.0"` を含む |

テスト数: 2378 → **2384**（+6）

---

## 完了条件

- [ ] `Cargo.toml` version = "30.2.0"
- [ ] `fav new --template postgres-etl my-proj` が 4 ソースファイル + tests/ + README.md を生成する
- [ ] `new_template_postgres_etl_creates_dir` テストが更新・通過する
- [ ] `scaffold_postgres_etl_uses_chain` テストが引き続き通過する
- [ ] `cargo test` — 2384 tests PASS
- [ ] `CHANGELOG.md` に `[v30.2.0]` セクションあり
- [ ] `benchmarks/v30.2.0.json` 存在（test_count: 2384）
- [ ] `cargo test --bin fav v302000` — 6/6 PASS
- [ ] `versions/current.md` を `v30.2.0` に更新
- [ ] tasks.md を COMPLETE に更新

---

## 検証コマンド

```bash
cd /c/Users/yoshi/favnir/fav

cargo test --bin fav v302000 2>&1 | tail -5
cargo test 2>&1 | grep "test result"
```
