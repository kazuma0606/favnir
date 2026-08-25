# Tasks: v87.4.0 — `create_sales_order()` + `NewSalesOrder`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,981 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87300_tests` が存在することを確認する（v87.3.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）

## T1: `runes/sap-odata/sales_order.fav` に `NewSalesOrder` 型 + `create_sales_order()` を追加

- [x] `public type NewSalesOrderItem` 型（`material_id`, `quantity`, `unit`）を定義する
- [x] `public type NewSalesOrder` 型（`customer_id`, `sales_org`, `currency`, `items`）を定義する
- [x] `public fn create_sales_order(cfg: SapConfig, order: NewSalesOrder) -> Result<SalesOrder, String>` を追加する（スタブ）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `sales_order_by_id()` ラッパーの直後に `NewSalesOrderItem` / `NewSalesOrder` 型の re-export を追加する
- [x] `create_sales_order()` ラッパー関数を追加する
- Note: T2 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは sales_order.fav のみ参照）

## T3: `driver.rs` に `mod v87400_tests` を追加

- [x] `mod v87300_tests { ... }` の直後に `#[cfg(test)] mod v87400_tests { ... }` を追加する
- [x] `create_sales_order_function_exists` テストを実装する
- [x] `new_sales_order_type_exists` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,983 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する

## 修正事項（code-reviewer 指摘対応）

- [MED] `create_business_partner` ラッパーが `body` パラメータ名を使うのに対し `create_sales_order` は `order` を使い混在している。ロードマップが `order` を指定しているため現実装はロードマップ準拠。将来 v88.x で `create_purchase_order` 等を追加する際は `order` 系で統一する方針とする。
- [LOW] `create_sales_order_function_exists` の `contains("fn create_sales_order")` → `contains("public fn create_sales_order(")` に変更（将来の `create_sales_order_batch` 等との衝突を防止）
- [LOW] `new_sales_order_type_exists` の `contains("NewSalesOrder")` → `contains("type NewSalesOrder =")` に変更（`NewSalesOrderItem` による誤検知を防止）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
