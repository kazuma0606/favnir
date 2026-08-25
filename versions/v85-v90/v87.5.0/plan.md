# Plan: v87.5.0 — シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）

## 実装ステップ

### Step 1: `runes/sap-odata/sales_order.fav` に集計型を追加

`create_sales_order()` の直後に以下を追加:

```favnir
-- 売上レポート集計型（v87.5.0）

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

### Step 2: `runes/sap-odata/sap_odata.fav` を更新

`create_sales_order()` ラッパーの直後に以下を追加:

```favnir
public type CurrencyTotal = sales_order.CurrencyTotal
public type SalesReport   = sales_order.SalesReport
```

### Step 3: `infra/e2e-demo/sap-odata/pipeline.fav` を更新

`sync_business_partners()` の直後に以下を追加:

```favnir
-- シナリオ 2: 日次売上レポート（SalesOrder 集計 → S3）（v87.5.0）
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

また `daily_sales_report()` が参照する `build_sales_report` スタブを同ファイルに追加する:

```favnir
-- ヘルパー関数スタブ（v87.7.0 で本実装）
fn build_sales_report(date: String, orders: List<SalesOrder>) -> Result<SalesReport, String> {
    Result.err("not implemented")
}
```

### Step 4: `fav/src/driver.rs` に `mod v87500_tests` を追加

`mod v87400_tests` の直後に以下を追加:

```rust
#[cfg(test)]
mod v87500_tests {
    #[test]
    fn sales_report_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/sales_order.fav")
            .expect("runes/sap-odata/sales_order.fav should exist");
        assert!(content.contains("type SalesReport ="), "SalesReport type should be defined");
    }
    #[test]
    fn sap_e2e_pipeline_contains_daily_sales_report() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("daily_sales_report"), "pipeline.fav should contain daily_sales_report");
    }
}
```

### Step 5: `cargo test` で全 pass 確認（3983 + 2 = 3985）
