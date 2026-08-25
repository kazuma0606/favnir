# Tasks: v87.3.0 — `sales_order_by_id()` + `$expand=to_Item`

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,979 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87200_tests` が存在することを確認する（v87.2.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）

## T1: `runes/sap-odata/sales_order.fav` に `sales_order_by_id()` を追加

- [x] `sales_orders()` 関数の直後に `-- 単一 SalesOrder 取得（v87.3.0）` コメントを追加する
- [x] `public fn sales_order_by_id(cfg: SapConfig, order_id: String, expand_items: Bool) -> Result<SalesOrder, String>` を追加する（スタブ）

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `sales_orders()` ラッパーの直後に `sales_order_by_id()` re-export を追加する
- Note: T2 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは sales_order.fav のみ参照）

## T3: `driver.rs` に `mod v87300_tests` を追加

- [x] `mod v87200_tests { ... }` の直後に `#[cfg(test)] mod v87300_tests { ... }` を追加する
- [x] `sales_order_by_id_function_exists` テストを実装する
- [x] `sales_order_expand_items_in_rune` テストを実装する

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,981 tests, 0 failures であることを確認する

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
