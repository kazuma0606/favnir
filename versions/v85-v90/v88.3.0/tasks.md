# Tasks: v88.3.0 — `PurchaseOrder` / `PurchaseOrderItem` 型定義

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,001 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88200_tests` が存在することを確認する（v88.2.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/types.fav` に発注伝票型を追加

- [x] `SapError` 型の前に `public type PurchaseOrderStatus = Open | PartiallyDelivered | Completed | Cancelled` を追加する
- [x] `public type PurchaseOrderItem = { ... }` を定義する（7 フィールド: `item_number` / `material_id` / `quantity` / `unit` / `net_price` / `currency` / `plant`）
- [x] `public type PurchaseOrder = { ... }` を定義する（7 フィールド: `po_number` / `vendor_id` / `status` / `total_amount` / `currency` / `created_at` / `items`）

## T2: `driver.rs` に `mod v88300_tests` を追加

- [x] `mod v88200_tests { ... }` の直後に `#[cfg(test)] mod v88300_tests { ... }` を追加する
- [x] `purchase_order_type_defined_in_rune` テストを実装する（`types.fav` で `"PurchaseOrder"` / `"PurchaseOrderStatus"` を確認）
- [x] `purchase_order_item_type_defined_in_rune` テストを実装する（`types.fav` で `"PurchaseOrderItem"` を確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,003 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] `purchase_order_type_defined_in_rune` テストに `PurchaseOrderStatus` の assert を追加（spec.md / plan.md / tasks.md 全修正）
- [MED] spec.md Files to Modify に `public type` は既存スタイルを踏襲する旨の注記を追加
