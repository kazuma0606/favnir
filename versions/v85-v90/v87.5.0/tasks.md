# Tasks: v87.5.0 — シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）

Status: COMPLETE

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,983 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87400_tests` が存在することを確認する（v87.4.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` が存在し `sync_business_partners` を含むことを確認する

## T1: `runes/sap-odata/sales_order.fav` に集計型を追加

- [x] `public type CurrencyTotal` 型（`currency`, `amount`, `count`）を定義する
- [x] `public type SalesReport` 型（`report_date`, `total_orders`, `total_amount`, `by_currency`）を定義する

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `create_sales_order()` ラッパーの直後に `CurrencyTotal` / `SalesReport` の re-export を追加する
- Note: T2 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは sales_order.fav と pipeline.fav を参照）

## T3: `infra/e2e-demo/sap-odata/pipeline.fav` に `daily_sales_report()` を追加

- [x] `sync_business_partners()` の直後に `-- シナリオ 2` コメントと `daily_sales_report()` 関数を追加する
- [x] `build_sales_report` スタブ（`fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String> { Result.err("not implemented") }`）を `pipeline.fav` に追加する（v87.7.0 で本実装）

## T4: `driver.rs` に `mod v87500_tests` を追加

- [x] `mod v87400_tests { ... }` の直後に `#[cfg(test)] mod v87500_tests { ... }` を追加する
- [x] `sales_report_type_exists` テストを実装する（`"type SalesReport ="` で検索）
- [x] `sap_e2e_pipeline_contains_daily_sales_report` テストを実装する

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,985 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する

## 修正事項（code-reviewer 指摘対応）

- [MED] `sales_report_type_exists` の `contains("type SalesReport =")` → `contains("public type SalesReport")` に変更（`public` 欠落も検知できるよう強化）
- [MED] `build_sales_report` スタブを `sales_order.fav` に置いているが、v87.7.0 では `runes/sap-odata/sales_report.fav`（新規ファイル）に本実装を置く予定。v87.7.0 実装時に `sales_order.fav` のスタブを削除して `sales_report.fav` に移動すること。
- [LOW] `sap_e2e_pipeline_contains_daily_sales_report` の `contains("daily_sales_report")` → `contains("fn daily_sales_report")` に変更（コメント内文字列による誤検知を防止）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
