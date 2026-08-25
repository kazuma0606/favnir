# Plan: v88.6.0 — シナリオ 3: 在庫 × 受注クロスチェック

## 実装ステップ

### Step 1: `runes/sap-odata/stock.fav` を新規作成

```favnir
-- 在庫不足アラート型定義（v88.6.0）
-- detect_stock_shortage() の実装は v88.7.0 で追加する

-- 在庫不足の深刻度
public type StockSeverity = Critical | Warning | Info

-- 在庫不足アラート型
public type StockAlert = {
    material_id:   String,
    description:   String,
    severity:      StockSeverity,
    open_quantity: Float,
    message:       String
}
```

### Step 2: `infra/e2e-demo/sap-odata/pipeline.fav` にシナリオ 3 を追加

`daily_sales_report` 関数の直後に追加:

```favnir
-- シナリオ 3: 在庫 × 受注クロスチェック（v88.6.0）

fn check_stock_vs_orders(ctx: AppCtx) -> Result<List<StockAlert>, String> {
    bind cfg       <- sap_odata.sap_config_from_env()
    bind orders    <- sap_odata.sales_orders(cfg, SalesOrderFilter {
        status:         Option.some(SalesOrderStatus.Open),
        customer_id:    Option.none(),
        created_after:  Option.none(),
        created_before: Option.none(),
        sales_org:      Option.none(),
        top:            Option.none()
    })
    bind materials <- sap_odata.materials(cfg, MaterialFilter {
        material_type: Option.some(MaterialType.FinishedProduct),
        plant:         Option.none(),
        top:           Option.none()
    })
    bind alerts    <- sap_odata.detect_stock_shortage(orders, materials)
    Result.ok(alerts)
}
```

### Step 3: `fav/src/driver.rs` に `mod v88600_tests` を追加

`mod v88500_tests { ... }` の直後に追加:

```rust
#[cfg(test)]
mod v88600_tests {
    #[test]
    fn sap_e2e_pipeline_contains_check_stock_vs_orders() {
        let content = std::fs::read_to_string("../infra/e2e-demo/sap-odata/pipeline.fav")
            .expect("infra/e2e-demo/sap-odata/pipeline.fav should exist");
        assert!(content.contains("check_stock_vs_orders"), "pipeline.fav should contain check_stock_vs_orders function");
    }
    #[test]
    fn stock_alert_type_exists() {
        let content = std::fs::read_to_string("../runes/sap-odata/stock.fav")
            .expect("runes/sap-odata/stock.fav should exist");
        assert!(content.contains("StockAlert"), "StockAlert should be defined in stock.fav");
    }
}
```

### Step 4: `cargo test` で全 pass 確認

4,007 + 2 = 4,009 tests, 0 failures を確認する。

---

**Note**: CHANGELOG / MILESTONE / site MDX 更新は v89.0.0 宣言バージョンでまとめて実施するため、本バージョンでは省略する。
