# v34.1.0 — 実装プラン

## 方針

実案件デモ追加パターン。`examples/real-world-etl/` を 8 ファイル構成で新規作成し、
`fav check` / `cargo test` が通ることを確認する。
`cargo clean` は x.1.0 のため不要。

---

## 実装ステップ

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の version を `34.0.0` → `34.1.0` に変更。

### Step 2: examples/real-world-etl/ 作成

以下の順序でファイルを作成する。

#### 2-1. fav.toml

```toml
[project]
name    = "real-world-etl"
version = "0.1.0"
edition = "2026"
src     = "src"

[postgres]
# url = "$DATABASE_URL"
sslmode = "require"

[runes]
csv      = { version = "1.0" }
postgres = { version = "1.0" }
bigquery = { version = "1.0" }
slack    = { version = "1.0" }
otel     = { version = "1.0" }
```

#### 2-2. src/types.fav

```favnir
type OrderStatus = | Pending | Processing | Shipped | Cancelled

type Order = {
    order_id:   String
    customer:   String
    product:    String
    quantity:   Int
    price:      Float
    status:     OrderStatus
    created_at: String
}

type ValidationError = {
    row:     Int
    field:   String
    message: String
}

type LoadResult = {
    orders: List<Order>
    errors: List<ValidationError>
}
```

#### 2-3. src/validators.fav

```favnir
fn validate_order(row: Int, o: Order) -> Result<Order, ValidationError> {
    if o.order_id == "" {
        Result.err({ row, field: "order_id", message: "order_id is required" })
    } else if o.quantity <= 0 {
        Result.err({ row, field: "quantity", message: "quantity must be positive" })
    } else if o.price < 0.0 {
        Result.err({ row, field: "price", message: "price must be non-negative" })
    } else {
        Result.ok(o)
    }
}

fn validate_all(orders: List<Order>) -> LoadResult {
    let pairs = List.indexed(orders)
    let results = List.map(pairs, fn (pair) {
        validate_order(pair.index, pair.value)
    })
    let valid_orders  = List.filter_map(results, fn (r) { Result.to_option(r) })
    let errors        = List.filter_map(results, fn (r) { Result.err_to_option(r) })
    { orders: valid_orders, errors }
}
```

#### 2-4. src/stages.fav

```favnir
import runes/csv
import runes/postgres
import runes/bigquery

fn load_csv(path: String) -> List<Order> !Io {
    let raw = Csv.read_file(path)
    List.map(raw, fn (row) {
        {
            order_id:   row["order_id"]
            customer:   row["customer"]
            product:    row["product"]
            quantity:   Int.parse(row["quantity"])
            price:      Float.parse(row["price"])
            status:     Pending
            created_at: row["created_at"]
        }
    })
}

fn write_postgres(orders: List<Order>) -> Int !Postgres {
    let rows = List.map(orders, fn (o) {
        [o.order_id, o.customer, o.product, Int.to_string(o.quantity), Float.to_string(o.price)]
    })
    Postgres.execute_raw(
        "INSERT INTO orders (order_id, customer, product, quantity, price) VALUES ($1,$2,$3,$4,$5)",
        rows
    )
}

fn sync_bigquery(orders: List<Order>) -> Int !Http {
    let payload = List.map(orders, fn (o) {
        { order_id: o.order_id, customer: o.customer, quantity: o.quantity, price: o.price }
    })
    BigQuery.insert("orders_dataset", "orders_table", payload)
}
```

#### 2-5. src/notifications.fav

```favnir
import runes/slack

fn notify_success(count: Int, errors: Int) -> Unit !Http {
    let msg = String.concat([
        ":white_check_mark: ETL completed. "
        "Rows loaded: " Int.to_string(count) ". "
        "Validation errors: " Int.to_string(errors) "."
    ])
    Slack.post_message("#data-pipeline", msg)
}

fn notify_failure(reason: String) -> Unit !Http {
    let msg = String.concat([":x: ETL failed: " reason])
    Slack.post_message("#data-pipeline", msg)
}
```

#### 2-6. src/main.fav

```favnir
import runes/otel

pipeline RealWorldEtl {
    stage LoadCsv {
        let raw = load_csv("data/orders_sample.csv")
        raw
    }
    |> stage Validate {
        let result = validate_all(input)
        result
    }
    |> stage WritePostgres {
        let n = write_postgres(input.orders)
        { written: n, errors: input.errors }
    }
    |> stage SyncBigQuery {
        let m = sync_bigquery(input.orders)
        { written: input.written, bq_written: m, errors: input.errors }
    }
    |> stage Notify {
        notify_success(input.written, List.length(input.errors))
        input
    }
}

fn main() -> Unit !Io !Postgres !Http {
    OTel.with_trace("real-world-etl", fn () {
        run RealWorldEtl
    })
}
```

#### 2-7. data/orders_sample.csv

```csv
order_id,customer,product,quantity,price,status,created_at
```

（ヘッダー行のみ。実データは README 参照。）

#### 2-8. README.md

30 分で動かす手順書。以下の構成:

```markdown
# real-world-etl

Favnir で構築した実案件規模の ETL パイプライン。

## 概要

S3 から CSV をロードし、バリデーションを行い、Postgres / BigQuery に書き込み、
Slack に通知し、OTel でトレースを記録する 5 ステージ構成のパイプラインです。

## 30 分で動かす手順

### 前提

- Favnir CLI がインストール済み（`fav --version` で確認）
- Docker Compose で Postgres が起動済み
- `.env` に環境変数を設定済み（`.env.example` 参照）

### セットアップ

1. リポジトリをクローン
2. `fav check` — 型チェックを実行
3. `DATABASE_URL=... fav run src/main.fav` — パイプラインを実行

## ファイル構成

...（spec.md 参照）
```

---

### Step 2-9: fav check 実行確認

```bash
cd /c/Users/yoshi/favnir/fav && ./target/debug/fav check ../examples/real-world-etl/src/main.fav
```

エラーなしを確認してから次ステップへ進む。

### Step 3: driver.rs 更新

1. `cargo_toml_version_is_34_0_0` を空スタブ化（他のテストは残存・スタブ化しない）
2. `v340000_tests` 直後・`// ── v31.7.0 tests` の前に `v341000_tests` を挿入

```rust
// ── v34.1.0 tests ────────────────────────────────────────────────────────────
#[cfg(test)]
mod v341000_tests {
    #[test]
    fn cargo_toml_version_is_34_1_0() {
        let src = include_str!("../Cargo.toml");
        assert!(src.contains("34.1.0"), "Cargo.toml must contain '34.1.0'");
    }

    #[test]
    fn real_world_etl_fav_toml_exists() {
        let src = include_str!("../../examples/real-world-etl/fav.toml");
        assert!(src.contains("real-world-etl"), "examples/real-world-etl/fav.toml must exist");
    }

    #[test]
    fn real_world_etl_readme_exists() {
        let src = include_str!("../../examples/real-world-etl/README.md");
        assert!(
            src.contains("30"),
            "examples/real-world-etl/README.md must mention '30' (30-minute setup)"
        );
    }

    #[test]
    fn real_world_etl_main_fav_exists() {
        let src = include_str!("../../examples/real-world-etl/src/main.fav");
        assert!(
            src.contains("pipeline") || src.contains("main"),
            "examples/real-world-etl/src/main.fav must exist"
        );
    }

    #[test]
    fn real_world_etl_sample_data_exists() {
        let src = include_str!("../../examples/real-world-etl/data/orders_sample.csv");
        assert!(
            src.contains("order_id"),
            "examples/real-world-etl/data/orders_sample.csv must have header 'order_id'"
        );
    }
}
```

挿入位置の確認コマンド:

```bash
grep -n "v340000_tests\|// ── v31.7.0" fav/src/driver.rs
```

### Step 4: CHANGELOG.md 更新

先頭に `[v34.1.0]` セクションを追加:

```markdown
## [v34.1.0] — 2026-07-04

### Added
- `examples/real-world-etl/` — 5 ファイル構成の実案件規模 ETL デモ
  - S3 CSV ロード → バリデーション → Postgres 書き込み → BigQuery 同期 → Slack 通知 → OTel トレース
  - 30 分で動かせる README 付き
```

### Step 5: benchmarks/v34.1.0.json 作成

```json
{
  "version": "34.1.0",
  "milestone": "Production Ready",
  "date": "2026-07-04",
  "tests_passed": 2541,
  "tests_failed": 0,
  "notes": "実案件デモ examples/real-world-etl/ 追加。v341000_tests 5件追加。"
}
```

（`tests_passed` は `cargo test` 実測後に確定）

### Step 6: versions/current.md 更新

最新安定版を v34.1.0 に変更。

---

## テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test --bin fav v341000 2>&1 | tail -8
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

---

## 完了処理

- `benchmarks/v34.1.0.json` の `tests_passed` を実測値で確定
- `tasks.md` を COMPLETE に更新
