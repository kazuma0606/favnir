# v34.1.0 — Spec

## 概要

**テーマ**: 実案件デモ実装

**方針**: `examples/real-world-etl/` に複数 Rune・複数ファイル構成の実規模パイプラインを追加する。
500 行以上・5 ファイル構成の ETL デモを通じて「Production Ready」への橋渡しとする。

---

## 背景

v34.0.0（Performance & Tooling 宣言）完了後の最初のスプリント。
ロードマップ `roadmap-v34.1-v35.0.md` の v34.1 計画に従い、実規模デモを `examples/real-world-etl/` として追加する。

`examples/csv-to-postgres/`（types.fav / stages.fav / validators.fav / main.fav）が既存デモとして存在するが、
v34.1 ではそれを大幅に拡張した「本番相当の ETL デモ」を新規ディレクトリとして追加する。

### ロードマップからの設計変更

| 項目 | ロードマップ定義 | 本 spec の判断 | 理由 |
|---|---|---|---|
| `orders_sample.csv` の行数 | 10,000 行 | ヘッダー行のみのスタブ | CI での `include_str!` テストに実データ不要。README に実データ生成手順を記載して代替 |
| Rust テスト件数 | 1 件（存在確認） | 5 件 | fav.toml / README.md / main.fav / orders_sample.csv の 4 ファイルを個別に網羅するため拡充 |

---

## 実装スコープ

### 追加ファイル一覧

```
examples/real-world-etl/
├── fav.toml                   プロジェクト定義
├── src/
│   ├── types.fav              注文データの型定義（Order / OrderStatus / ValidationError）
│   ├── validators.fav         ビジネスルールのバリデーション（欠損値・範囲・重複）
│   ├── stages.fav             ETL ステージ群（LoadCsv / Validate / WritePostgres / SyncBigQuery）
│   ├── notifications.fav      Slack / Email 通知（notify_success / notify_failure）
│   └── main.fav               エントリポイント（pipeline 定義 + OTel トレース）
├── data/
│   └── orders_sample.csv      サンプルデータヘッダー行（実データ参照用スタブ）
└── README.md                  30 分で動かす手順書
```

### 変更ファイル

1. `fav/Cargo.toml` — version `34.0.0` → `34.1.0`
2. `fav/src/driver.rs` — `cargo_toml_version_is_34_0_0` をスタブ化、`v341000_tests` 5 件追加
3. `benchmarks/v34.1.0.json` — 新規作成
4. `CHANGELOG.md` — `[v34.1.0]` セクション先頭追記
5. `versions/current.md` — 最新安定版を v34.1.0 に更新

---

## テスト仕様（v341000_tests）

```rust
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

### 設計注記

- `use super::*` なし（`include_str!` のみ使用）
- WASM ゲートなし（ファイル読み込みのみ）
- v341000_tests は v340000_tests 直後・`// ── v31.7.0 tests` の前に挿入

---

## examples/real-world-etl/ コンテンツ仕様

### src/types.fav

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

### src/validators.fav

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

### src/stages.fav

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

### src/notifications.fav

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

### src/main.fav

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

### data/orders_sample.csv（ヘッダー行のみ）

```csv
order_id,customer,product,quantity,price,status,created_at
```

---

## fav.toml 仕様

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

---

## 完了条件

- [ ] `cargo clean` 不要（x.1.0 はクリーンアップ不要、x.0.0 のみ必須）
- [ ] `Cargo.toml` version = `"34.1.0"`
- [ ] `cargo_toml_version_is_34_0_0` が空スタブ（他テストは残存・スタブ化しない）
- [ ] `fav check examples/real-world-etl/src/main.fav` — エラーなし
- [ ] `cargo test --bin fav v341000` — 5/5 PASS
- [ ] `cargo test` — 全件 PASS（2541 件想定 = v34.0.0 の 2536 + v341000_tests 5 件、0 failures）
- [ ] `examples/real-world-etl/` が 8 ファイル（fav.toml + src/ 5 ファイル + data/ 1 ファイル + README.md）で構成されていること
- [ ] `examples/real-world-etl/README.md` に「30 分で動かす」手順が書かれていること
- [ ] `CHANGELOG.md` に `[v34.1.0]` セクション
- [ ] `benchmarks/v34.1.0.json` 存在かつ `tests_passed` が実測値
- [ ] `versions/current.md` を v34.1.0 に更新
