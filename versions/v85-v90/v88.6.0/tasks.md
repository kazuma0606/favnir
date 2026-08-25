# Tasks: v88.6.0 — シナリオ 3: 在庫 × 受注クロスチェック

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、4,007 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v88500_tests` が存在することを確認する（v88.5.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `88.0.0` であることを確認する（宣言バージョン v88.0.0 以降はスプリント中も 88.0.0 のまま）

## T1: `runes/sap-odata/stock.fav` を新規作成

- [x] `public type StockSeverity = Critical | Warning | Info` を定義する
- [x] `public type StockAlert = { ... }` を定義する（5 フィールド: `material_id` / `description` / `severity` / `open_quantity` / `message`）
- Note: `detect_stock_shortage()` の実装は v88.7.0 で追加する

## T2: `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 3 を追加

- [x] `daily_sales_report` 関数の直後に `-- シナリオ 3` コメントと `fn check_stock_vs_orders(ctx: AppCtx) -> Result<List<StockAlert>, String>` を追加する
- [x] 関数内で `sap_odata.sap_config_from_env()` / `sap_odata.sales_orders()` / `sap_odata.materials()` / `sap_odata.detect_stock_shortage()` を呼び出す

## T3: `driver.rs` に `mod v88600_tests` を追加

- [x] `mod v88500_tests { ... }` の直後に `#[cfg(test)] mod v88600_tests { ... }` を追加する
- [x] `sap_e2e_pipeline_contains_check_stock_vs_orders` テストを実装する（`pipeline.fav` で `"check_stock_vs_orders"` を確認）
- [x] `stock_alert_type_exists` テストを実装する（`stock.fav` で `"StockAlert"` を確認）

## T4: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、4,009 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する

## spec-reviewer 指摘対応

- [HIGH] ロードマップ v88.7.0 の `stock.fav`「新規作成」→「v88.6.0 で先行作成済み・本バージョンでは追記」に修正
- [MED] spec.md テスト数をベースライン明示形式（T0 確認値 4,007 + 2 = 4,009）に変更
