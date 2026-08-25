# Tasks: v87.8.0 — モックサーバーテスト（受注シナリオ全操作検証）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,989 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87700_tests` が存在することを確認する（v87.7.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）
- [x] `runes/sap-odata/sap_odata.test.fav` に BusinessPartner CRUD テスト（v86.7.0）が存在することを確認する（追記の起点）

## T1: `runes/sap-odata/sap_odata.test.fav` にテストを追加

- [x] `business_partner_list` テストの直後に SalesOrder CRUD テスト 3 件を追加する:
  - `test "sales_order_create"` — `create_sales_order` シグネチャ確認コメント付き
  - `test "sales_order_read"` — `sales_order_by_id` シグネチャ確認コメント付き
  - `test "sales_order_filter"` — `sales_orders` フィルタシグネチャ確認コメント付き
- [x] ページネーションテスト 1 件を追加する:
  - `test "pagination_over_100_items"` — `odata_list_paged` シグネチャ確認コメント付き
- [x] 日次売上レポートテスト 1 件を追加する:
  - `test "daily_sales_report_pipeline"` — `build_sales_report` シグネチャ確認コメント付き

## T2: `driver.rs` に `mod v87800_tests` を追加

- [x] `mod v87700_tests { ... }` の直後に `#[cfg(test)] mod v87800_tests { ... }` を追加する
- [x] `sap_odata_test_contains_sales_order_tests` テストを実装する（`sap_odata.test.fav` で `"sales_order_create"` を確認）
- [x] `sap_odata_test_contains_pagination_test` テストを実装する（`sap_odata.test.fav` で `"pagination_over_100_items"` を確認）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,991 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
