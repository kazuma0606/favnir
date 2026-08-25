# Tasks: v88.5.0 — `create_purchase_order()` POST 実装

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,005 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88400_tests` が存在することを確認する（v88.4.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/purchase_order.fav` に型と関数を追加

- [x] `purchase_order_by_id()` の直後に `public type NewPurchaseOrderItem = { ... }` を定義する（4 フィールド: `material_id` / `quantity` / `unit` / `plant`）
- [x] `public type NewPurchaseOrder = { ... }` を定義する（3 フィールド: `vendor_id` / `currency` / `items`）
- [x] `public fn create_purchase_order(cfg: SapConfig, order: NewPurchaseOrder) -> Result<PurchaseOrder, String>` スタブを追加する（`Result.err("not implemented")` 返し）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `purchase_order_by_id()` ラッパーの直後に `NewPurchaseOrderItem` / `NewPurchaseOrder` 型エイリアスと `create_purchase_order()` ラッパーを追加する
- Note: T2 は手作業確認（Rust テストの対象外）

## T3: `driver.rs` に `mod v88500_tests` を追加

- [x] `mod v88400_tests { ... }` の直後に `#[cfg(test)] mod v88500_tests { ... }` を追加する
- [x] `create_purchase_order_function_exists` テストを実装する（`purchase_order.fav` で `"public fn create_purchase_order("` を確認）
- [x] `new_purchase_order_type_exists` テストを実装する（`purchase_order.fav` で `"NewPurchaseOrder"` / `"NewPurchaseOrderItem"` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,007 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
