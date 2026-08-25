# Tasks: v88.4.0 — `purchase_orders()` / `purchase_order_by_id()` クエリ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,003 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88300_tests` が存在することを確認する（v88.3.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/purchase_order.fav` を新規作成

- [x] ファイル先頭に `use sap_odata.types` を追加する
- [x] `public type PurchaseOrderFilter = { ... }` を定義する（5 フィールド: `vendor_id` / `status` / `created_after` / `plant` / `top`）
- [x] `public fn purchase_orders(cfg: SapConfig, filter: PurchaseOrderFilter) -> Result<List<PurchaseOrder>, String>` スタブを追加する（`Result.err("not implemented")` 返し）
- [x] `public fn purchase_order_by_id(cfg: SapConfig, po_number: String, expand_items: Bool) -> Result<PurchaseOrder, String>` スタブを追加する（`Result.err("not implemented")` 返し）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `use sap_odata.material` の直後に `use sap_odata.purchase_order` を追加する
- [x] `material_by_id()` ラッパーの直後に `PurchaseOrderStatus` / `PurchaseOrderItem` / `PurchaseOrder` 型エイリアスと `PurchaseOrderFilter` 型エイリアス、`purchase_orders()` / `purchase_order_by_id()` ラッパーを追加する
- Note: T2 は手作業確認（Rust テストの対象外）
- Note: `purchase_order_by_id` 関数の存在も手作業で確認する（ロードマップ定義のテスト 2 件に含まれないため）

## T3: `driver.rs` に `mod v88400_tests` を追加

- [x] `mod v88300_tests { ... }` の直後に `#[cfg(test)] mod v88400_tests { ... }` を追加する
- [x] `purchase_orders_function_exists` テストを実装する（`purchase_order.fav` で `"public fn purchase_orders("` を確認）
- [x] `purchase_order_filter_type_exists` テストを実装する（`purchase_order.fav` で `"PurchaseOrderFilter"` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,005 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [MED] `purchase_order_by_id` は手作業確認項目に明示（ロードマップ 2 件テストの方針維持）
- [MED] `sap_odata.fav` re-export がテスト対象外である理由を正確に記載（re-export 欠落でもテストが通ることを認識した上でスコープ外）
