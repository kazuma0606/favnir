# Spec: v87.7.0 — `SalesReport` 集計型 + `group_by_currency()`

## Background

v87.5.0 で `SalesReport` / `CurrencyTotal` 型と `daily_sales_report()` パイプラインを追加した。
その際、`build_sales_report()` は `pipeline.fav` 内のスタブとして暫定定義した（"v87.7.0 で本実装"コメント付き）。

本バージョンでは売上集計ロジックを `runes/sap-odata/sales_report.fav` に分離・本実装し、
`pipeline.fav` のスタブを削除して `sap_odata.build_sales_report()` 呼び出しに切り替える。

## Goals

1. `runes/sap-odata/sales_report.fav` を新規作成し、売上集計ヘルパー 3 関数を実装する
2. `group_by_currency()` で通貨別に受注を集計する内部ロジックを実装する（スタブ可）
3. `build_sales_report()` を `public fn` として定義し、`sap_odata.fav` 経由で外部公開する
4. `format_sales_report()` を追加してレポートの文字列表現を返す（スタブ可）
5. `pipeline.fav` のローカルスタブを削除し、`sap_odata.build_sales_report()` に切り替える

## API / Syntax Examples

```favnir
-- runes/sap-odata/sales_report.fav

use sap_odata.types
use sap_odata.sales_order

-- 内部ヘルパー: 通貨別集計
fn group_by_currency(orders: List<SalesOrder>) -> List<CurrencyTotal> {
    List.empty()
}

-- 日次売上レポート生成
public fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String> {
    Result.ok(SalesReport {
        report_date:  date,
        total_orders: List.length(orders),
        total_amount: 0.0,
        by_currency:  group_by_currency(orders)
    })
}

-- レポートをテキスト形式にフォーマット（public fn: sap_odata.fav re-export のため）
public fn format_sales_report(report: SalesReport) -> String {
    String.concat(["Sales Report ", report.report_date, ": ", Int.to_string(report.total_orders), " orders"])
}
```

```favnir
-- pipeline.fav（更新後）
-- build_sales_report スタブを削除し sap_odata 経由に変更

fn daily_sales_report(ctx: AppCtx) -> Result<SalesReport, String> {
    bind cfg    <- sap_odata.sap_config_from_env()
    bind orders <- sap_odata.sales_orders(cfg, SalesOrderFilter { ... })
    bind report <- sap_odata.build_sales_report("2026-08-22", orders)
    bind json   <- Json.encode(report)
    bind _      <- ctx.s3.put_object("favnir-sap-demo", "reports/daily/2026-08-22.json", json)
    Result.ok(report)
}
```

## Success Criteria（Rust テストで担保）

- `runes/sap-odata/sales_report.fav` が存在し、以下を含む:
  - `fn group_by_currency(`（内部ヘルパー）
  - `public fn build_sales_report(`
  - `public fn format_sales_report(`（re-export に必要なため `public fn`）
- `cargo test` で 3,989 tests, 0 failures
- Rust テスト 2 件: `group_by_currency_function_exists` / `format_sales_report_function_exists`

## 手作業確認項目（Rust テスト対象外）

- `runes/sap-odata/sap_odata.fav` が `use sap_odata.sales_report` を含み、`build_sales_report` / `format_sales_report` を re-export している（T2）
- `infra/e2e-demo/sap-odata/pipeline.fav` のローカルスタブ `fn build_sales_report` が削除されている（T3）

## Files to Modify

| ファイル | 変更種別 |
|---|---|
| `runes/sap-odata/sales_report.fav` | **新規作成** |
| `runes/sap-odata/sap_odata.fav` | 追記（`use` + re-export 2 件） |
| `infra/e2e-demo/sap-odata/pipeline.fav` | スタブ削除・呼び出し更新 |
| `fav/src/driver.rs` | `mod v87700_tests` 追加 |
