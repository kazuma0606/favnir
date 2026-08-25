# Spec: v87.5.0 — シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）

## Background

v87.4.0 で `create_sales_order()` POST 実装が完成した。
本バージョンでは業務シナリオ 2（日次売上レポート）の E2E 実装を行う。

`SalesReport` / `CurrencyTotal` 集計型を定義し、
`daily_sales_report()` 関数を `infra/e2e-demo/sap-odata/pipeline.fav` に追加する。
シナリオ 1（BusinessPartner → S3）と同じファイルに追記する形式。

## Goals

- `runes/sap-odata/sales_order.fav` に `SalesReport` / `CurrencyTotal` 型を追加する
- `infra/e2e-demo/sap-odata/pipeline.fav` に `daily_sales_report()` 関数を追加する
- `sap_odata.fav` に `SalesReport` / `CurrencyTotal` の re-export を追加する
- Rust テスト 2 件で型・パイプライン関数の存在を確認する

## Syntax / API

```favnir
-- runes/sap-odata/sales_order.fav に追加（v87.5.0）

public type CurrencyTotal = {
    currency: String,
    amount:   Float,
    count:    Int
}

public type SalesReport = {
    report_date:  String,
    total_orders: Int,
    total_amount: Float,
    by_currency:  List<CurrencyTotal>
}
```

```favnir
-- infra/e2e-demo/sap-odata/pipeline.fav に追加（v87.5.0）

-- シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）
fn daily_sales_report(ctx: AppCtx) -> Result<SalesReport, String> {
    bind cfg    <- sap_odata.sap_config_from_env()
    bind orders <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status:        Option.some(SalesOrderStatus.Completed),
        created_after: Option.some("2026-08-22"),
        customer_id:   Option.none(),
        created_before: Option.none(),
        sales_org:     Option.none(),
        top:           Option.none()
    })
    bind report <- build_sales_report("2026-08-22", orders)
    bind json   <- Json.encode(report)
    bind _      <- ctx.s3.put_object("favnir-sap-demo", "reports/daily/2026-08-22.json", json)
    Result.ok(report)
}
```

Note: `build_sales_report` は v87.7.0 で本実装する。
本バージョンでは以下のスタブを `pipeline.fav` に追加し `daily_sales_report()` が参照できるようにする:

```favnir
-- ヘルパー関数スタブ（v87.7.0 で本実装）
fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String> {
    Result.err("not implemented")
}
```

## Success Criteria

1. `runes/sap-odata/sales_order.fav` に `SalesReport` 型が定義されている
2. `infra/e2e-demo/sap-odata/pipeline.fav` に `daily_sales_report` 関数が含まれている
3. `pipeline.fav` に `build_sales_report` スタブ（`fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String>`）が追加されている
4. `runes/sap-odata/sap_odata.fav` に `SalesReport` / `CurrencyTotal` の re-export が追加されている
5. Rust テスト 2 件が pass: `sales_report_type_exists` / `sap_e2e_pipeline_contains_daily_sales_report`
6. `cargo test` 全 pass（3983 + 2 = 3985 tests）

## Files to Modify

- `runes/sap-odata/sales_order.fav` — `CurrencyTotal` / `SalesReport` 型を追加
- `runes/sap-odata/sap_odata.fav` — `CurrencyTotal` / `SalesReport` re-export を追加
- `infra/e2e-demo/sap-odata/pipeline.fav` — `daily_sales_report()` 関数を追加
- `fav/src/driver.rs` — `mod v87500_tests` 追加
