# Tasks: v87.9.0 — 安定化・コードフリーズ

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,991 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87800_tests` が存在することを確認する（v87.8.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）
- [x] 本バージョンはバグ修正のみ（新機能追加なし）であることを確認する

## T1: `driver.rs` に `mod v87900_tests` を追加

- [x] `mod v87800_tests { ... }` の直後に `#[cfg(test)] mod v87900_tests { ... }` を追加する
- [x] `sap_sales_order_crud_covered` テストを実装する（`sales_order.fav` に CRUD 3 関数が揃っていることを確認）:
  - `"public fn sales_orders("`
  - `"public fn sales_order_by_id("`
  - `"public fn create_sales_order("`
- [x] `sap_sales_scenario2_report_pipeline_exists` テストを実装する（`pipeline.fav` に `"sap_odata.build_sales_report("` が含まれることを確認）

## T2: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,993 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
