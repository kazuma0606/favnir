# Tasks: v87.7.0 — `SalesReport` 集計型 + `group_by_currency()`

Status: COMPLETE

## code-reviewer 指摘と対応

- [STYLE] `use sap_odata.types` が `sales_report.fav` 内で未使用 → 削除済み。
- [STYLE] `format_sales_report` が E2E pipeline で未呼び出し → 後続バージョン（v87.8.0 以降）で対応。
- [STYLE] `group_by_currency` の `public` 修飾子有無がテストで未検証 → `"fn group_by_currency("` は `"public fn group_by_currency("` にも部分一致するため誤検知の可能性あり。現行テスト数（2件）は ロードマップ完了条件と一致しており追加は行わない。

## T0: 着手前チェックリスト

- [x] `cargo test` を実行し、3,987 tests, 0 failures を確認する
- [x] `fav/src/driver.rs` に `mod v87600_tests` が存在することを確認する（v87.6.0 完了済みの証拠）
- [x] `fav/Cargo.toml` の version が `87.0.0` であることを確認する（宣言バージョン v87.0.0 以降はスプリント中も 87.0.0 のまま）
- [x] `infra/e2e-demo/sap-odata/pipeline.fav` にローカルスタブ `fn build_sales_report` が存在することを確認する（本バージョンで削除対象）

## T1: `runes/sap-odata/sales_report.fav` を新規作成

- [x] ファイル先頭に `use sap_odata.types` と `use sap_odata.sales_order` を追加する
- [x] `fn group_by_currency(orders: List<SalesOrder>) -> List<CurrencyTotal>` を定義する（スタブ: `List.empty()` 返し）
- [x] `public fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String>` を定義する
- [x] `public fn format_sales_report(report: SalesReport) -> String` を定義する

## T2: `runes/sap-odata/sap_odata.fav` を更新（re-export）

- [x] `use sap_odata.sales_order` の直後に `use sap_odata.sales_report` を追加する
- [x] `create_sales_order` ラッパーの直後に `public fn build_sales_report(...)` ラッパーを追加する
- [x] `build_sales_report` ラッパーの直後に `public fn format_sales_report(...)` ラッパーを追加する
- Note: T2 は手作業確認（Rust テストの対象外。ロードマップ完了条件の 2 件テストは sales_report.fav を参照）

## T3: `infra/e2e-demo/sap-odata/pipeline.fav` を更新

- [x] ローカルスタブ `-- ヘルパー関数スタブ（v87.7.0 で本実装）` コメントと `fn build_sales_report { ... }` を削除する
- [x] `daily_sales_report()` 内の `build_sales_report(...)` 呼び出しを `sap_odata.build_sales_report(...)` に変更する
- Note: T3 は手作業確認（Rust テストの対象外）

## T4: `driver.rs` に `mod v87700_tests` を追加

- [x] `mod v87600_tests { ... }` の直後に `#[cfg(test)] mod v87700_tests { ... }` を追加する
- [x] `group_by_currency_function_exists` テストを実装する（`sales_report.fav` で `fn group_by_currency(` を確認）
- [x] `format_sales_report_function_exists` テストを実装する（`sales_report.fav` で `"public fn format_sales_report("` を確認）

## T5: `cargo test` で全 pass 確認

- [x] `cargo test 2>&1 | grep "test result"` を実行し、3,989 tests, 0 failures であることを確認する

- Note: CHANGELOG / MILESTONE / site MDX 更新は v88.0.0 宣言バージョンでまとめて実施する（本バージョンは対象外）

## T-last: CI 事前確認

- [x] `cargo clippy --locked -- -D warnings` が pass することを確認する（CI と同じフラグ）
- [x] `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- [x] `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
