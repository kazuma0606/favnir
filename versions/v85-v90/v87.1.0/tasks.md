# Tasks: v87.1.0 — `SalesOrder` / `SalesOrderItem` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,975 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87000_tests` が存在することを確認する（v87.0.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する

## T1: `runes/sap-odata/sales_order.fav` を新規作成

- [x] `SalesOrderStatus = Open | InProcess | Completed | Cancelled` を定義する
- [x] `SalesOrderItem` レコード型（`item_number`, `material_id`, `description`, `quantity`, `unit`, `net_amount`, `currency`）を定義する
- [x] `SalesOrder` レコード型（`order_id`, `customer_id`, `status`, `total_amount`, `currency`, `sales_org`, `created_at`, `items`）を定義する

## T2: `driver.rs` に `mod v87100_tests` を追加

- [x] `mod v87000_tests { ... }` の直後に `#[cfg(test)] mod v87100_tests { ... }` を追加する
- [x] `sales_order_type_defined_in_rune` テストを実装する（`order_id` フィールドの存在確認）
- [x] `sales_order_item_type_defined_in_rune` テストを実装する（`item_number` フィールドの存在確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,977 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## 修正事項（code-reviewer 指摘対応）

- [BUG] code-reviewer が `roadmap-v87.1-v88.0.md`（実在しないファイル）を参照して「types.fav に追記すべき」と指摘したが、実際のロードマップ（roadmap-v85.1-v90.0.md）には実装ファイルの指定なし。`business_partner.fav` と同パターンのエンティティ別ファイル（`sales_order.fav`）が正しい設計であることを確認。
- [MED] `use sap_odata.types` が v87.1.0 時点で未使用（SapConfig 参照は v87.2.0 以降）→ 削除し、コメントで v87.2.0 で追加する旨を明記
- [LOW] テストアサーションにフィールド名のみでなく型名（`SalesOrder` / `SalesOrderItem`）の確認も追加
