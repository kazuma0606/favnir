# Tasks: v95.5.0 — Deep Insert

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `versions/current.md` の最新安定版が `v95.4.0` であることを確認する
- [x] `infra/e2e-demo/sap-odata/pipeline_realtime.fav` が存在することを確認する（v95.4.0 完了済みの証拠）
- [x] `fav/src/driver.rs` に `mod v95400_tests` が存在することを確認する（v95.4.0 完了済みの証拠）
- [x] `cargo test 2>&1 | grep "test result"` を実行し、現在のテスト数が 4,172 であることを確認する（着手前ベースライン）
- [x] `fav/Cargo.toml` の version が `95.0.0` であることを確認する
  （スプリント内バージョンでは Cargo.toml を更新しない）

## T1: `sales_order.fav` に Deep Insert 型と関数を追加

- [x] `runes/sap-odata/sales_order.fav` の末尾に `NewSalesOrderWithItems` 型を追加する
  （customer_id: String, currency: String, items: List<NewSalesOrderItem>）
- [x] `create_sales_order_deep` 関数スタブを追加する
  （`cfg: SapConfig, order: NewSalesOrderWithItems) -> Result<SalesOrder, String>`、戻り値 `Result.err("not implemented")`）
- [x] 既存の `NewSalesOrderItem` フィールド名（`material_id`）との整合性を確認する

## T2: `driver.rs` にテストを追加

- [x] `mod v95400_tests` の直後に `#[cfg(test)] mod v95500_tests { ... }` を追加する
- [x] `deep_insert_type_defined` テストを追加する（`sales_order.fav` に `NewSalesOrderWithItems` が含まれる）
- [x] `create_sales_order_deep_defined` テストを追加する（`sales_order.fav` に `create_sales_order_deep` が含まれる）

## T3: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,174 tests, 0 failures であることを確認する

## T4: CHANGELOG / current.md 更新

- [x] `CHANGELOG.md` の先頭に `[v95.5.0]` エントリを追加する
- [x] `versions/current.md` の最新安定版を `v95.5.0` に更新する

## T5: tasks.md 更新

- [x] 本ファイルの Status を `COMPLETE` に変更する
- [x] T0〜T-last の全チェックボックスを `[x]` にする

## T-last: CI 事前確認（T3 の `cargo test` 全 pass 確認後に実施すること）

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
